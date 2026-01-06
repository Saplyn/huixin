use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use cpal::traits::{DeviceTrait, StreamTrait};
use egui_snarl::{InPinId, NodeId, OutPinId};
use log::{debug, info, trace, warn};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::Topo,
};

use crate::model::{
    patch::{
        Block, Number, Patch, PatchOutput,
        node::{
            PatchNode, PatchNodeTrait, PatchNodeType,
            number::NumberNode,
            oscillator::{Oscillator, Waveform},
            speaker::Speaker,
        },
    },
    state::{CentralState, PatchId},
};

const SLEEP_DURATION: Duration = Duration::from_millis(10);

pub enum Command {
    RebuildGraph,
}

type DiGraphNode = (PatchId, NodeId);
type DiGraphEdge = (usize /* from pin */, usize /* to pin */);

pub fn main(state: Arc<CentralState>, cmd_rx: mpsc::Receiver<Command>) -> ! {
    let mut graph: DiGraph<DiGraphNode, DiGraphEdge> = DiGraph::new();

    let (output_tx, output_rx) = mpsc::sync_channel::<Block>(1);

    let stream = state
        .cpal
        .device
        .build_output_stream(
            &state.cpal.config,
            move |output: &mut [f32], _| {
                let block = output_rx.recv().unwrap();
                for (frame, samp) in output.iter_mut().zip(block) {
                    *frame = samp as f32;
                }
            },
            move |err| {
                info!("{:?}", err);
            },
            None,
        )
        .unwrap();
    stream.play().unwrap();

    loop {
        match cmd_rx.try_recv() {
            Ok(Command::RebuildGraph) => {
                rebuild_graph(&state, &mut graph);
            }
            Err(mpsc::TryRecvError::Empty) => (),
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("Processor command channel disconnected");
            }
        }

        let mut output = [PatchOutput::empty_block(); 2];
        let mut topo = Topo::new(&graph);
        while let Some(node_index) = topo.next(&graph) {
            let (ref patch_id, node_id) = graph[node_index];
            let patch_arc = state.get_patch(patch_id).unwrap();
            let mut patch_guard = patch_arc.write();

            match patch_guard.snarl[node_id].get_type() {
                // Signal
                PatchNodeType::Oscillator => {
                    let PatchNode::Oscillator(osc) = &patch_guard.snarl[node_id] else {
                        unreachable!();
                    };

                    let mut freq = None;
                    if let Some(src) = osc.input_for_pin(Oscillator::INPUT_FREQ) {
                        freq = patch_guard.snarl[src.node]
                            .output_number(src.output)
                            .map(|n| {
                                n.clamp(
                                    *Oscillator::FREQ_RANGE.start(),
                                    *Oscillator::FREQ_RANGE.end(),
                                )
                            });
                    }

                    let mut phase = None;
                    if let Some(src) = osc.input_for_pin(Oscillator::INPUT_PHASE) {
                        phase = patch_guard.snarl[src.node]
                            .output_number(src.output)
                            .map(|n| {
                                n.clamp(
                                    *Oscillator::PHASE_RANGE.start(),
                                    *Oscillator::PHASE_RANGE.end(),
                                )
                            });
                    }

                    let mut waveform = None;
                    if let Some(src) = osc.input_for_pin(Oscillator::INPUT_WAVEFORM) {
                        waveform = patch_guard.snarl[src.node]
                            .output_number(src.output)
                            .map(|n| {
                                n.clamp(
                                    *Oscillator::WAVEFORM_RANGE.start() as Number,
                                    *Oscillator::WAVEFORM_RANGE.end() as Number,
                                ) as usize
                            });
                    }

                    let PatchNode::Oscillator(osc) = &mut patch_guard.snarl[node_id] else {
                        unreachable!();
                    };
                    if let Some(freq_or_seed) = freq {
                        osc.freq_or_seed = freq_or_seed;
                    }
                    if let Some(phase) = phase {
                        osc.phase = phase;
                    }
                    if let Some(waveform) = waveform {
                        osc.waveform = Waveform::from(waveform);
                    }
                    osc.next_block(state.cpal.config.sample_rate);
                }
                PatchNodeType::Speaker => {
                    let PatchNode::Speaker(speaker) = &mut patch_guard.snarl[node_id] else {
                        unreachable!();
                    };
                    let sources = speaker
                        .inputs_for_pin(Speaker::INPUT_LEFT_CHAN)
                        .iter()
                        .copied()
                        .collect::<Vec<_>>();
                    for src in sources {
                        let block = patch_guard.snarl[src.node]
                            .output_block(src.output)
                            .unwrap();
                        output[0].iter_mut().zip(block).for_each(|(frame, samp)| {
                            *frame += *samp;
                        });
                    }
                }

                // Variable
                PatchNodeType::Number => {
                    let PatchNode::Number(num) = &patch_guard.snarl[node_id] else {
                        unreachable!();
                    };

                    let mut val = None;
                    if let Some(num) = num.input_for_pin(NumberNode::INPUT_NUM) {
                        val = patch_guard.snarl[num.node].output_number(num.output);
                    }

                    let PatchNode::Number(num) = &mut patch_guard.snarl[node_id] else {
                        unreachable!();
                    };
                    if let Some(v) = val {
                        num.number = v;
                    }
                }
            }
        }

        output_tx.send(output[0]).unwrap();

        thread::sleep(SLEEP_DURATION);
    }
}

fn rebuild_graph(state: &CentralState, graph: &mut DiGraph<DiGraphNode, DiGraphEdge>) {
    graph.clear();
    build_graph(state, graph);
}

fn build_graph(state: &CentralState, graph: &mut DiGraph<DiGraphNode, DiGraphEdge>) {
    let mut node_id_to_index: HashMap<(PatchId, NodeId), NodeIndex> = HashMap::new();

    for entry in state.patches_iter() {
        let patch_id = entry.key().clone();
        let Patch { snarl, .. } = &*entry.read();

        for (node_id, _) in snarl.node_ids() {
            let index = graph.add_node((patch_id.clone(), node_id));
            node_id_to_index.insert((patch_id.clone(), node_id), index);
        }

        for (from_pin, to_pin) in snarl.wires() {
            let OutPinId {
                node: from_node_id,
                output: from_pin,
            } = from_pin;
            let from_patch_id = patch_id.clone();

            let InPinId {
                node: to_node_id,
                input: to_pin,
            } = to_pin;
            let to_patch_id = patch_id.clone();

            let from_index = node_id_to_index
                .get(&(from_patch_id, from_node_id))
                .expect("From node not found in index map");
            let to_index = node_id_to_index
                .get(&(to_patch_id, to_node_id))
                .expect("To node not found in index map");
            graph.add_edge(*from_index, *to_index, (from_pin, to_pin));
        }
    }
}

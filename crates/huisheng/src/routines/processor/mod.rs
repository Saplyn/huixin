use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use cpal::{
    FromSample, I24, Sample, SizedSample, U24,
    traits::{DeviceTrait, StreamTrait},
};
use egui_snarl::{InPinId, NodeId, OutPinId};
use log::{debug, info, trace, warn};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::Topo,
};

use crate::model::{
    patch::{
        BLOCK_SIZE, Block, Number, Patch, PatchOutput,
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

    let device = &state.cpal.device;
    let format = state.cpal.supported_config.sample_format();
    let config = state.cpal.supported_config.config();
    let sample_rate = config.sample_rate;

    let (output_tx, output_rx) = mpsc::sync_channel::<[Number; 2]>(0);

    let stream = build_stream(device, format, config, output_rx);
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
                // Signal/Oscillator
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
                    osc.next_block(sample_rate);
                }

                // Signal/Speaker
                PatchNodeType::Speaker => {
                    let PatchNode::Speaker(speaker) = &mut patch_guard.snarl[node_id] else {
                        unreachable!();
                    };

                    let left_chan_src = speaker
                        .inputs_for_pin(Speaker::INPUT_LEFT_CHAN)
                        .iter()
                        .copied()
                        .collect::<Vec<_>>();
                    let right_chan_src = speaker
                        .inputs_for_pin(Speaker::INPUT_RIGHT_CHAN)
                        .iter()
                        .copied()
                        .collect::<Vec<_>>();

                    for src in left_chan_src {
                        let block = patch_guard.snarl[src.node]
                            .output_block(src.output)
                            .unwrap();
                        output[0].iter_mut().zip(block).for_each(|(frame, samp)| {
                            *frame += *samp;
                        });
                    }
                    for src in right_chan_src {
                        let block = patch_guard.snarl[src.node]
                            .output_block(src.output)
                            .unwrap();
                        output[1].iter_mut().zip(block).for_each(|(frame, samp)| {
                            *frame += *samp;
                        });
                    }
                }

                // Variable/Number
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

        for (left_sample, right_sample) in output[0].into_iter().zip(output[1]) {
            output_tx.send([left_sample, right_sample]).unwrap();
        }

        thread::sleep(SLEEP_DURATION);
    }
}

// LYN: Stream Building

fn build_stream(
    device: &cpal::Device,
    format: cpal::SampleFormat,
    config: cpal::StreamConfig,
    output_rx: mpsc::Receiver<[Number; 2]>,
) -> cpal::Stream {
    match format {
        cpal::SampleFormat::I8 => build_stream_inner::<i8>(device, config, output_rx),
        cpal::SampleFormat::I16 => build_stream_inner::<i16>(device, config, output_rx),
        cpal::SampleFormat::I24 => build_stream_inner::<I24>(device, config, output_rx),
        cpal::SampleFormat::I32 => build_stream_inner::<i32>(device, config, output_rx),
        cpal::SampleFormat::I64 => build_stream_inner::<i64>(device, config, output_rx),

        cpal::SampleFormat::U8 => build_stream_inner::<u8>(device, config, output_rx),
        cpal::SampleFormat::U16 => build_stream_inner::<u16>(device, config, output_rx),
        cpal::SampleFormat::U24 => build_stream_inner::<U24>(device, config, output_rx),
        cpal::SampleFormat::U32 => build_stream_inner::<u32>(device, config, output_rx),
        cpal::SampleFormat::U64 => build_stream_inner::<u64>(device, config, output_rx),

        cpal::SampleFormat::F32 => build_stream_inner::<f32>(device, config, output_rx),
        cpal::SampleFormat::F64 => build_stream_inner::<f64>(device, config, output_rx),

        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

fn build_stream_inner<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    output_rx: mpsc::Receiver<[Number; 2]>,
) -> cpal::Stream
where
    T: SizedSample + FromSample<Number>,
{
    device
        .build_output_stream(
            &config,
            move |output: &mut [T], _| {
                write_stream(output, &output_rx, config.channels as usize);
            },
            move |err| {
                info!("{:?}", err);
            },
            None,
        )
        .unwrap()
}

fn write_stream<T>(output: &mut [T], output_rx: &mpsc::Receiver<[Number; 2]>, channels: usize)
where
    T: Sample + FromSample<Number>,
{
    for frame in output.chunks_mut(channels) {
        let [left_sample, right_sample] = output_rx.recv().unwrap();
        if channels == 2 {
            frame[0] = T::from_sample(left_sample);
            frame[1] = T::from_sample(right_sample);
        } else {
            let val = T::from_sample((left_sample + right_sample) / 2.);
            frame.iter_mut().for_each(|sample| {
                *sample = val;
            });
        }
    }
}

// LYN: Graph Building

#[inline]
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

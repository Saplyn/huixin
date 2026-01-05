use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use cpal::traits::{DeviceTrait, StreamTrait};
use egui_snarl::{InPinId, NodeId, OutPinId};
use log::{debug, info, trace, warn};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::model::{
    patch::{
        Block, Patch, PatchOutput,
        node::{PatchNode, speaker::Speaker},
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
    let mut order = Vec::new();
    let mut data: HashMap<(PatchId, NodeId), Block> = HashMap::new();

    let (output_tx, output_rx) = mpsc::sync_channel::<Block>(1);

    let stream = state
        .cpal
        .device
        .build_output_stream(
            &state.cpal.config,
            move |output: &mut [f32], info| {
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
                order = petgraph::algo::toposort(&graph, None).expect("Graph has cycles");
            }
            Err(mpsc::TryRecvError::Empty) => (),
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("Processor command channel disconnected");
            }
        }

        let mut output = [PatchOutput::empty_block(); 2];
        data.clear();
        for node_index in &order {
            let (ref patch_id, node_id) = graph[*node_index];
            let arc = state.get_patch(patch_id).unwrap();
            let mut guard = arc.write();

            match &mut guard.snarl[node_id] {
                PatchNode::Oscillator(osc) => {
                    let block = osc.next_block(48000.);
                    data.insert((patch_id.clone(), node_id), block);
                }
                PatchNode::Speaker(speaker) => {
                    for src in speaker.inputs_for_pin(Speaker::INPUT_LEFT_CHAN) {
                        if let Some(block) = data.get(&(patch_id.clone(), src.node)) {
                            output[0].iter_mut().zip(block).for_each(|(frame, samp)| {
                                *frame += *samp;
                            });
                        } else {
                            warn!("No data for left channel input from node {:?}", src.node);
                        }
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

// pub fn orig_main(
//     state: Arc<CentralState>,
//     receiver_rx: mpsc::Receiver<(LynId, mpsc::Receiver<Block>)>,
//     frame_rx: mpsc::Receiver<()>,
// ) -> ! {
//     info!("Processer started");
//     let mut block_rx_set: HashMap<LynId, mpsc::Receiver<Block>> = HashMap::new();
//     let (output_tx, output_rx) = mpsc::channel();
//
//     let channels = state.cpal.supported_config.channels() as usize;
//     let stream = state
//         .cpal
//         .device
//         .build_output_stream(
//             &state.cpal.config,
//             move |output: &mut [f32], info| {
//                 info!("{:#?}", info.timestamp());
//                 info!("{info:#?}");
//                 for frame in output {
//                     let value = output_rx.recv().unwrap();
//                     *frame = value;
//                 }
//                 // thread::sleep(Duration::from_secs_f64(0.5));
//             },
//             move |err| {
//                 info!("{:?}", err);
//             },
//             None,
//         )
//         .unwrap();
//     stream.play().unwrap();
//
//     loop {
//         while let Ok((id, block_rx)) = receiver_rx.try_recv() {
//             block_rx_set.insert(id, block_rx);
//         }
//
//         frame_rx.recv().unwrap();
//
//         let mut block = [0.; BLOCK_SIZE];
//         let mut to_be_removed = Vec::new();
//         for (&id, block_rx) in block_rx_set.iter() {
//             let received = match block_rx.try_recv() {
//                 Ok(block) => block,
//                 Err(mpsc::TryRecvError::Empty) => continue,
//                 Err(mpsc::TryRecvError::Disconnected) => {
//                     to_be_removed.push(id);
//                     continue;
//                 }
//             };
//
//             for (frame, received_frame) in block.iter_mut().zip(received) {
//                 *frame += received_frame;
//             }
//         }
//         for id in to_be_removed {
//             block_rx_set.remove(&id);
//         }
//
//         for sample in block {
//             output_tx.send(sample as f32);
//         }
//     }
// }

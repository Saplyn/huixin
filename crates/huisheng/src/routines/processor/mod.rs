use std::sync::{Arc, mpsc};

use cpal::traits::StreamTrait;
use egui_snarl::NodeId;
use petgraph::{graph::DiGraph, visit::Topo};

use crate::{
    model::{
        patch::{
            Number, WireDataType,
            node::{
                PatchNode, PatchNodeTrait, PatchNodeType, bang::BangNode, number::NumberNode,
                oscillator::Oscillator, remote_data::RemoteData, speaker::Speaker,
            },
        },
        state::{CentralState, PatchId},
    },
    routines::processor::{graph::rebuild_graph, node::PatchNodeProcessable, stream::build_stream},
};

mod graph;
mod node;
mod stream;

pub enum Command {
    RebuildGraph,
}

type DiGraphNode = (PatchId, NodeId);
type DiGraphEdge = (usize /* from pin */, usize /* to pin */);

const BUFFER_SIZE: usize = 64;

pub fn main(state: Arc<CentralState>, cmd_rx: mpsc::Receiver<Command>) -> ! {
    let mut graph: DiGraph<DiGraphNode, DiGraphEdge> = DiGraph::new();

    let device = &state.cpal.device;
    let format = state.cpal.supported_config.sample_format();
    let config = state.cpal.supported_config.config();
    let sample_rate = config.sample_rate;

    let (output_tx, output_rx) = mpsc::sync_channel::<[Number; 2]>(BUFFER_SIZE);

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

        let mut request_repaint = false;
        let mut discard = false;

        // Topological process
        let mut output = [WireDataType::empty_block(); 2];
        let mut topo = Topo::new(&graph);
        while let Some(node_index) = topo.next(&graph) {
            let (ref patch_id, node_id) = graph[node_index];
            let patch_arc = state.sheet_get_patch(patch_id).unwrap();
            let mut patch_guard = patch_arc.write();

            let Some(node) = patch_guard.snarl.get_node_mut(node_id) else {
                discard = true;
                break;
            };
            node.pre_process(state.clone());
            let process_result = match node.get_type() {
                // Signal
                PatchNodeType::Oscillator => {
                    Oscillator::process(node_id, &mut patch_guard.snarl, sample_rate)
                }
                PatchNodeType::Speaker => {
                    Speaker::process(node_id, &mut patch_guard.snarl, &mut output)
                }

                // Variable
                PatchNodeType::Number => NumberNode::process(node_id, &mut patch_guard.snarl, ()),
                PatchNodeType::Bang => BangNode::process(node_id, &mut patch_guard.snarl, ()),

                // Communication
                PatchNodeType::RemoteData => {
                    RemoteData::process(node_id, &mut patch_guard.snarl, ())
                }
            };
            if let Ok(repaint) = process_result {
                request_repaint |= repaint;
            } else {
                discard = true;
                break;
            }
        }

        // Post process
        for node_idx in graph.node_indices() {
            let (ref patch_id, node_id) = graph[node_idx];
            let patch_arc = state.sheet_get_patch(patch_id).unwrap();
            let mut patch_guard = patch_arc.write();
            patch_guard.snarl[node_id].post_process();
        }

        // Handle potential repaint
        if request_repaint && let Some(ctx) = state.ui.ctx.get() {
            ctx.request_repaint();
        }

        // Discard this pass on error
        if discard {
            continue;
        }

        // Send output block to audio stream
        for (left_sample, right_sample) in output[0].into_iter().zip(output[1]) {
            output_tx.send([left_sample, right_sample]).unwrap();
        }
    }
}

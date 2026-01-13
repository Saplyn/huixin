use std::collections::HashMap;

use egui_snarl::{InPinId, NodeId, OutPinId};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::{
    model::{
        patch::Patch,
        state::{CentralState, PatchId},
    },
    routines::processor::{DiGraphEdge, DiGraphNode},
};

#[inline]
pub fn rebuild_graph(state: &CentralState, graph: &mut DiGraph<DiGraphNode, DiGraphEdge>) {
    graph.clear();
    build_graph(state, graph);
}

fn build_graph(state: &CentralState, graph: &mut DiGraph<DiGraphNode, DiGraphEdge>) {
    let mut node_id_to_index: HashMap<(PatchId, NodeId), NodeIndex> = HashMap::new();

    for entry in state.sheet_patches_iter() {
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

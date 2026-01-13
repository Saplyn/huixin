use egui_snarl::{NodeId, Snarl};

use crate::model::patch::node::PatchNode;

pub mod bang;
pub mod number;
pub mod oscillator;
pub mod remote_data;
pub mod speaker;

pub trait PatchNodeProcessable<'arg> {
    type ProcessArg;
    fn process(node_id: NodeId, snarl: &mut Snarl<PatchNode>, arg: Self::ProcessArg) -> bool;
}

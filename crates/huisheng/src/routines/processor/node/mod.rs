use egui_snarl::{NodeId, Snarl};

use crate::model::patch::node::PatchNode;

pub mod bang;
pub mod midi_to_freq;
pub mod number;
pub mod oscillator;
pub mod remote_data;
pub mod speaker;

pub trait PatchNodeProcessable<'arg> {
    type ProcessArg;
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        arg: Self::ProcessArg,
    ) -> Result<bool, ()>;
}

#[macro_export]
macro_rules! node_or_bail {
    ($snarl:expr, $node:expr) => {
        $snarl.get_node($node).ok_or(())?
    };
    (mut $snarl:expr, $node:expr) => {
        $snarl.get_node_mut($node).ok_or(())?
    };
}

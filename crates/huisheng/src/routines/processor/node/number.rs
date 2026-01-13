use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::node::{PatchNode, PatchNodeTrait, number::NumberNode},
    routines::processor::node::PatchNodeProcessable,
};

impl PatchNodeProcessable<'_> for NumberNode {
    type ProcessArg = ();
    fn process(node_id: NodeId, snarl: &mut Snarl<PatchNode>, _: Self::ProcessArg) -> bool {
        let PatchNode::Number(num) = &snarl[node_id] else {
            unreachable!();
        };

        let mut val = None;
        if let Some(num) = num.input_for_pin(NumberNode::INPUT_NUM) {
            val = snarl[num.node].output_number(num.output);
        }

        let PatchNode::Number(num) = &mut snarl[node_id] else {
            unreachable!();
        };
        if let Some(v) = val {
            num.number = v;
        }

        false
    }
}

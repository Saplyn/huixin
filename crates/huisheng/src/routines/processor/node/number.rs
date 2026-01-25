use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::node::{PatchNode, PatchNodeTrait, number::NumberNode},
    node_or_bail,
    routines::processor::node::PatchNodeProcessable,
};

impl PatchNodeProcessable<'_> for NumberNode {
    type ProcessArg = ();
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        _: Self::ProcessArg,
    ) -> Result<bool, ()> {
        let PatchNode::Number(num) = node_or_bail!(snarl, node_id) else {
            unreachable!();
        };

        let mut val = None;
        if let Some(num) = num.input_for_pin(NumberNode::INPUT_NUM) {
            val = node_or_bail!(snarl, num.node).output_number(num.output);
        }

        let PatchNode::Number(num) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };
        if let Some(v) = val {
            num.number = v;
        }

        Ok(false)
    }
}

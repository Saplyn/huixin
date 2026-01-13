use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::node::{PatchNode, PatchNodeTrait, bang::BangNode},
    routines::processor::node::PatchNodeProcessable,
};

impl PatchNodeProcessable<'_> for BangNode {
    type ProcessArg = ();
    fn process(node_id: NodeId, snarl: &mut Snarl<PatchNode>, _: Self::ProcessArg) -> bool {
        let PatchNode::Bang(bang) = &snarl[node_id] else {
            unreachable!();
        };

        let mut val = None;
        if let Some(src) = bang.input_for_pin(BangNode::INPUT_ARBITRARY).to_owned() {
            val = snarl[src.node].output_arbitrary(src.output, node_id);
        }

        let PatchNode::Bang(bang) = &mut snarl[node_id] else {
            unreachable!();
        };
        if val.is_some() {
            bang.update(val);
        }

        false
    }
}

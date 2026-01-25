use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::node::{PatchNode, PatchNodeTrait, bang::BangNode},
    node_or_bail,
    routines::processor::node::PatchNodeProcessable,
};

impl PatchNodeProcessable<'_> for BangNode {
    type ProcessArg = ();
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        _: Self::ProcessArg,
    ) -> Result<bool, ()> {
        let PatchNode::Bang(bang) = node_or_bail!(snarl, node_id) else {
            unreachable!();
        };

        let mut val = None;
        if let Some(src) = bang.input_for_pin(BangNode::INPUT_ARBITRARY).to_owned() {
            val = node_or_bail!(mut snarl, src.node).output_arbitrary(src.output, node_id);
        }

        let PatchNode::Bang(bang) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };
        if val.is_some() {
            bang.update(val);
        }

        Ok(false)
    }
}

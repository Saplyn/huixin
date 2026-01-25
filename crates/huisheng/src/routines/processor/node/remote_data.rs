use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::node::{PatchNode, PatchNodeTrait, remote_data::RemoteData},
    node_or_bail,
    routines::processor::node::PatchNodeProcessable,
};

impl PatchNodeProcessable<'_> for RemoteData {
    type ProcessArg = ();
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        _: Self::ProcessArg,
    ) -> Result<bool, ()> {
        let PatchNode::RemoteData(remote) = node_or_bail!(snarl, node_id) else {
            unreachable!();
        };

        let mut tag = None;
        if let Some(src) = remote.input_for_pin(RemoteData::INPUT_TAG) {
            tag = node_or_bail!(snarl, src.node).output_text(src.output);
        }
        let mut prop = None;
        if let Some(src) = remote.input_for_pin(RemoteData::INPUT_PROP) {
            prop = node_or_bail!(snarl, src.node).output_text(src.output);
        }

        let PatchNode::RemoteData(remote) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };

        if let Some(tag) = tag {
            remote.tag = tag;
        }
        if let Some(prop) = prop {
            remote.prop = prop;
        }

        Ok(remote
            .output_bang(RemoteData::OUTPUT_BANG, node_id)
            .is_some_and(|bang| bang))
    }
}

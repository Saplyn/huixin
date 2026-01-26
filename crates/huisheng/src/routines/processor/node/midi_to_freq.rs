use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::node::{PatchNode, PatchNodeTrait, midi_to_freq::MidiToFreq},
    node_or_bail,
    routines::processor::node::PatchNodeProcessable,
};

impl PatchNodeProcessable<'_> for MidiToFreq {
    type ProcessArg = ();
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        _: Self::ProcessArg,
    ) -> Result<bool, ()> {
        let PatchNode::MidiToFreq(mtf) = node_or_bail!(snarl, node_id) else {
            unreachable!();
        };

        let mut val = None;
        if let Some(num) = mtf.input_for_pin(MidiToFreq::INPUT_MIDI) {
            val = node_or_bail!(snarl, num.node).output_number(num.output);
        }

        let PatchNode::MidiToFreq(num) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };
        if let Some(v) = val {
            num.midi = v as u8;
        }

        Ok(false)
    }
}

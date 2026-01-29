use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::{
        Number,
        node::{PatchNode, PatchNodeTrait, adsr_curve::ADSRCurve},
    },
    node_or_bail,
    routines::processor::node::PatchNodeProcessable,
};

impl PatchNodeProcessable<'_> for ADSRCurve {
    type ProcessArg = u32;
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        sample_rate: Self::ProcessArg,
    ) -> Result<bool, ()> {
        let PatchNode::ADSRCurve(adsr) = node_or_bail!(snarl, node_id) else {
            unreachable!();
        };

        let mut attack = None;
        if let Some(num) = adsr.input_for_pin(ADSRCurve::INPUT_ATTACK) {
            attack = node_or_bail!(snarl, num.node).output_number(num.output);
        }

        let mut decay = None;
        if let Some(num) = adsr.input_for_pin(ADSRCurve::INPUT_DECAY) {
            decay = node_or_bail!(snarl, num.node).output_number(num.output);
        }

        let mut sustain = None;
        if let Some(num) = adsr.input_for_pin(ADSRCurve::INPUT_SUSTAIN) {
            sustain = node_or_bail!(snarl, num.node).output_number(num.output);
        }

        let mut release = None;
        if let Some(num) = adsr.input_for_pin(ADSRCurve::INPUT_RELEASE) {
            release = node_or_bail!(snarl, num.node).output_number(num.output);
        }

        let mut peak = None;
        if let Some(num) = adsr.input_for_pin(ADSRCurve::INPUT_PEAK) {
            peak = node_or_bail!(snarl, num.node).output_number(num.output);
        }

        let mut keep = None;
        if let Some(num) = adsr.input_for_pin(ADSRCurve::INPUT_PEAK) {
            keep = node_or_bail!(snarl, num.node).output_number(num.output);
        }

        let mut retrigger = None;
        if let Some(src) = adsr.input_for_pin(ADSRCurve::INPUT_TRIGGER) {
            retrigger = node_or_bail!(mut snarl, src.node).output_bang(src.output, node_id);
        }

        let PatchNode::ADSRCurve(adsr) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };
        if let Some(v) = attack {
            adsr.attack = v;
        }
        if let Some(v) = decay {
            adsr.decay = v;
        }
        if let Some(v) = sustain {
            adsr.sustain = v;
        }
        if let Some(v) = release {
            adsr.release = v;
        }
        if let Some(v) = peak {
            adsr.peak = v;
        }
        if let Some(v) = keep {
            adsr.keep = v;
        }
        if retrigger.is_some_and(|val| val) {
            adsr.trigger();
        }

        let blk = adsr.next_block(sample_rate);
        log::error!("{:?}", blk.get(1));

        Ok(false)
    }
}

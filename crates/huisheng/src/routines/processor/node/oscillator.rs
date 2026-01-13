use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::{
        Number,
        node::{
            PatchNode, PatchNodeTrait,
            oscillator::{Oscillator, Waveform},
        },
    },
    routines::processor::node::PatchNodeProcessable,
};

impl PatchNodeProcessable<'_> for Oscillator {
    type ProcessArg = u32;
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        sample_rate: Self::ProcessArg,
    ) -> bool {
        let PatchNode::Oscillator(osc) = &snarl[node_id] else {
            unreachable!();
        };

        let mut freq = None;
        if let Some(src) = osc.input_for_pin(Oscillator::INPUT_FREQ) {
            freq = snarl[src.node].output_number(src.output).map(|n| {
                n.clamp(
                    *Oscillator::FREQ_RANGE.start(),
                    *Oscillator::FREQ_RANGE.end(),
                )
            });
        }

        let mut phase = None;
        if let Some(src) = osc.input_for_pin(Oscillator::INPUT_PHASE) {
            phase = snarl[src.node].output_number(src.output).map(|n| {
                n.clamp(
                    *Oscillator::PHASE_RANGE.start(),
                    *Oscillator::PHASE_RANGE.end(),
                )
            });
        }

        let mut waveform = None;
        if let Some(src) = osc.input_for_pin(Oscillator::INPUT_WAVEFORM) {
            waveform = snarl[src.node].output_number(src.output).map(|n| {
                n.clamp(
                    *Oscillator::WAVEFORM_RANGE.start() as Number,
                    *Oscillator::WAVEFORM_RANGE.end() as Number,
                ) as usize
            });
        }

        let mut reset = None;
        if let Some(src) = osc.input_for_pin(Oscillator::INPUT_RESET) {
            reset = snarl[src.node].output_bang(src.output, node_id);
        }

        let PatchNode::Oscillator(osc) = &mut snarl[node_id] else {
            unreachable!();
        };
        if let Some(freq_or_seed) = freq {
            osc.freq_or_seed = freq_or_seed;
        }
        if let Some(phase) = phase {
            osc.phase = phase;
        }
        if let Some(waveform) = waveform {
            osc.waveform = Waveform::from(waveform);
        }
        if let Some(reset) = reset
            && reset
        {
            osc.reset();
        }
        osc.next_block(sample_rate);

        false
    }
}

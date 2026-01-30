use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::{
        WireDataType,
        node::{PatchNode, PatchNodeTrait, amp_multiplier::AmpMultiplier},
    },
    node_or_bail,
    routines::processor::node::PatchNodeProcessable,
};

impl<'output> PatchNodeProcessable<'output> for AmpMultiplier {
    type ProcessArg = ();
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        _: Self::ProcessArg,
    ) -> Result<bool, ()> {
        let PatchNode::AmpMultiplier(amp_mul) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };

        let left_ops = amp_mul
            .inputs_for_pin(AmpMultiplier::INPUT_LEFT_OPS)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let right_ops = amp_mul
            .inputs_for_pin(AmpMultiplier::INPUT_RIGHT_OPS)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>();

        let mut left_blocks = WireDataType::empty_block();
        for src in left_ops {
            if let Some(block) = node_or_bail!(snarl, src.node).output_block(src.output) {
                left_blocks.iter_mut().zip(block).for_each(|(frame, samp)| {
                    *frame += *samp;
                });
            }
        }

        let mut right_blocks = WireDataType::empty_block();
        for src in right_ops {
            if let Some(block) = node_or_bail!(snarl, src.node).output_block(src.output) {
                right_blocks
                    .iter_mut()
                    .zip(block)
                    .for_each(|(frame, samp)| {
                        *frame += *samp;
                    });
            }
        }

        let PatchNode::AmpMultiplier(amp_mul) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };
        amp_mul
            .memory_mut()
            .iter_mut()
            .zip(left_blocks.iter())
            .zip(right_blocks.iter())
            .for_each(|((out_frame, left_samp), right_samp)| {
                *out_frame = *left_samp * *right_samp;
            });

        Ok(false)
    }
}

use egui_snarl::{NodeId, Snarl};

use crate::{
    model::patch::{
        Block,
        node::{PatchNode, PatchNodeTrait, speaker::Speaker},
    },
    node_or_bail,
    routines::processor::node::PatchNodeProcessable,
};

impl<'output> PatchNodeProcessable<'output> for Speaker {
    type ProcessArg = &'output mut [Block; 2];
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        output: Self::ProcessArg,
    ) -> Result<bool, ()> {
        let PatchNode::Speaker(speaker) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };

        let left_chan_src = speaker
            .inputs_for_pin(Speaker::INPUT_LEFT_CHAN)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let right_chan_src = speaker
            .inputs_for_pin(Speaker::INPUT_RIGHT_CHAN)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>();

        for src in left_chan_src {
            let block = node_or_bail!(snarl, src.node)
                .output_block(src.output)
                .unwrap();
            output[0].iter_mut().zip(block).for_each(|(frame, samp)| {
                *frame += *samp;
            });
        }
        for src in right_chan_src {
            let block = node_or_bail!(snarl, src.node)
                .output_block(src.output)
                .unwrap();
            output[1].iter_mut().zip(block).for_each(|(frame, samp)| {
                *frame += *samp;
            });
        }

        Ok(false)
    }
}

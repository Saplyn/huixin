use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::patch_viewer::{
        NodeType,
        constants::{input_pin, output_pin},
    },
    model::patch::node::{PatchNode, bang::BangNode},
};

// LYN: Public Interface

impl BangNode {
    #[inline(always)]
    pub fn pin_input(
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::INPUT_ARBITRARY => input_arbitrary(pin, ui, snarl),
            _ => unreachable!("bang node only has {} inputs", Self::INPUTS),
        }
    }

    #[inline(always)]
    pub fn pin_output(
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::OUTPUT_BANG => output_bang(pin, ui, snarl),
            _ => unreachable!("bang node only has {} outputs", Self::OUTPUTS),
        }
    }
}

// LYN: Private Impl

type This = BangNode;

fn input_arbitrary(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    let PatchNode::Bang(bang) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    if ui.button("  ").clicked() {
        bang.update(None);
    }

    input_pin(
        This::INPUT_TYPE[This::INPUT_ARBITRARY],
        This::INPUT_ACCEPT_MULTI[This::INPUT_ARBITRARY],
    )
}

fn output_bang(
    pin: &egui_snarl::OutPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    output_pin(This::OUTPUT_TYPE[This::OUTPUT_BANG])
}

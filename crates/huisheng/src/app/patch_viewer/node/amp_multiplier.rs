use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::patch_viewer::{
        NodeType,
        constants::{input_pin, output_pin},
    },
    model::patch::node::amp_multiplier::AmpMultiplier,
};

// LYN: Public Interface

impl AmpMultiplier {
    #[inline(always)]
    pub fn pin_input(
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::INPUT_LEFT_OPS => input_left_ops(pin, ui, snarl),
            Self::INPUT_RIGHT_OPS => input_right_ops(pin, ui, snarl),
            _ => unreachable!("amp multiplier only has {} inputs", Self::INPUTS),
        }
    }

    #[inline]
    pub fn pin_output(
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::OUTPUT_RESULT => output_result(pin, ui, snarl),
            _ => unreachable!("amp multiplier only has {} outputs", Self::OUTPUTS),
        }
    }
}

// LYN: Private Impl

type This = AmpMultiplier;

fn input_left_ops(
    _pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    _snarl: &mut Snarl<crate::model::patch::node::PatchNode>,
) -> PinInfo {
    ui.label("左算子");

    input_pin(
        This::INPUT_TYPE[This::INPUT_LEFT_OPS],
        This::INPUT_ACCEPT_MULTI[This::INPUT_LEFT_OPS],
    )
}

fn input_right_ops(
    _pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    _snarl: &mut Snarl<crate::model::patch::node::PatchNode>,
) -> PinInfo {
    ui.label("右算子");

    input_pin(
        This::INPUT_TYPE[This::INPUT_RIGHT_OPS],
        This::INPUT_ACCEPT_MULTI[This::INPUT_RIGHT_OPS],
    )
}

fn output_result(
    _pin: &egui_snarl::OutPin,
    _ui: &mut egui::Ui,
    _snarl: &mut Snarl<crate::model::patch::node::PatchNode>,
) -> PinInfo {
    output_pin(This::OUTPUT_TYPE[This::OUTPUT_RESULT])
}

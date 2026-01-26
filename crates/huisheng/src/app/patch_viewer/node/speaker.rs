use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::patch_viewer::{NodeType, constants::input_pin},
    model::patch::node::speaker::Speaker,
};

// LYN: Public Interface

impl Speaker {
    #[inline(always)]
    pub fn pin_input(
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::INPUT_LEFT_CHAN => input_left_chan(pin, ui, snarl),
            Self::INPUT_RIGHT_CHAN => input_right_chan(pin, ui, snarl),
            _ => unreachable!("speaker only has {} inputs", Self::INPUTS),
        }
    }

    #[inline]
    pub fn pin_output(
        _pin: &egui_snarl::OutPin,
        _ui: &mut egui::Ui,
        _snarl: &mut Snarl<NodeType>,
        _index: usize,
    ) -> PinInfo {
        unreachable!("speaker has {} outputs", Self::OUTPUTS);
    }
}

// LYN: Private Impl

#[inline(always)]
fn input_left_chan(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
) -> PinInfo {
    ui.label("左声道");

    input_pin(
        Speaker::INPUT_TYPE[Speaker::INPUT_LEFT_CHAN],
        Speaker::INPUT_ACCEPT_MULTI[Speaker::INPUT_LEFT_CHAN],
    )
}

#[inline(always)]
fn input_right_chan(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
) -> PinInfo {
    ui.label("右声道");

    input_pin(
        Speaker::INPUT_TYPE[Speaker::INPUT_RIGHT_CHAN],
        Speaker::INPUT_ACCEPT_MULTI[Speaker::INPUT_RIGHT_CHAN],
    )
}

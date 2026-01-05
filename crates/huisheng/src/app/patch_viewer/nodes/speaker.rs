use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::patch_viewer::{NodeType, constants::input_pin},
    model::patch::node::speaker::Speaker,
};

// LYN: Public Interface

#[inline(always)]
pub fn speaker_input(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
    index: usize,
) -> PinInfo {
    match index {
        Speaker::INPUT_LEFT_CHAN => speaker_input_left_chan(pin, ui, snarl),
        Speaker::INPUT_RIGHT_CHAN => speaker_input_right_chan(pin, ui, snarl),
        _ => unreachable!("speaker only has {} inputs", Speaker::INPUTS),
    }
}

#[inline]
pub fn speaker_output(
    _pin: &egui_snarl::OutPin,
    _ui: &mut egui::Ui,
    _snarl: &mut Snarl<NodeType>,
    _index: usize,
) -> PinInfo {
    unreachable!("speaker has {} outputs", Speaker::OUTPUTS);
}

// LYN: Private Impl

#[inline(always)]
fn speaker_input_left_chan(
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
fn speaker_input_right_chan(
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

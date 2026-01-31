use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::patch_viewer::{
        NodeType,
        constants::{input_pin, output_pin},
    },
    model::patch::node::{PatchNode, remote_data::RemoteData},
};

// LYN: Public Interface

impl RemoteData {
    #[inline(always)]
    pub fn pin_input(
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::INPUT_TAG => input_tag(pin, ui, snarl),
            Self::INPUT_PROP => input_prop(pin, ui, snarl),
            _ => unreachable!("remote data only has {} inputs", Self::INPUTS),
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
            Self::OUTPUT_DATA => output_data(pin, ui, snarl),
            Self::OUTPUT_BANG => output_bang(pin, ui, snarl),
            _ => unreachable!("remote data only has {} outputs", Self::OUTPUTS),
        }
    }
}

// LYN: Private Impl

type This = RemoteData;

fn input_tag(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<PatchNode>) -> PinInfo {
    let PatchNode::RemoteData(remote) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("标签");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(80., 0.),
            egui::TextEdit::singleline(&mut remote.tag),
        );
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_TAG],
        This::INPUT_ACCEPT_MULTI[This::INPUT_TAG],
    )
}

fn input_prop(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<PatchNode>) -> PinInfo {
    let PatchNode::RemoteData(remote) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("属性");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(80., 0.),
            egui::TextEdit::singleline(&mut remote.prop),
        );
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_PROP],
        This::INPUT_ACCEPT_MULTI[This::INPUT_PROP],
    )
}

fn output_data(
    _pin: &egui_snarl::OutPin,
    ui: &mut egui::Ui,
    _snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    ui.label("数据");

    output_pin(This::OUTPUT_TYPE[This::OUTPUT_DATA])
}

fn output_bang(
    _pin: &egui_snarl::OutPin,
    ui: &mut egui::Ui,
    _snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    ui.label("触发");

    output_pin(This::OUTPUT_TYPE[This::OUTPUT_BANG])
}

use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::{
        helpers::WidgetId,
        patch_viewer::{
            NodeType,
            constants::{input_pin, output_pin},
        },
    },
    model::patch::{
        Number,
        node::{
            PatchNode,
            number::NumberNode,
            oscillator::{Oscillator, Waveform},
        },
    },
};

// LYN: Public Interface

impl NumberNode {
    #[inline(always)]
    pub fn pin_input(
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::INPUT_NUM => input_num(pin, ui, snarl),
            _ => unreachable!("number node only has {} inputs", Self::INPUTS),
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
            Self::OUTPUT_NUM => output_num(pin, ui, snarl),
            _ => unreachable!("number node only has {} outputs", Self::OUTPUTS),
        }
    }
}

// LYN: Private Impl

type This = NumberNode;

fn input_num(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<PatchNode>) -> PinInfo {
    let PatchNode::Number(num) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("数值");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(emath::vec2(60., 0.), egui::DragValue::new(&mut num.number));
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_NUM],
        This::INPUT_ACCEPT_MULTI[This::INPUT_NUM],
    )
}

fn output_num(
    pin: &egui_snarl::OutPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    output_pin(This::OUTPUT_TYPE[This::OUTPUT_NUM])
}

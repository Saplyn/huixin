use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::patch_viewer::{
        NodeType,
        constants::{input_pin, output_pin},
    },
    model::patch::node::{PatchNode, midi_to_freq::MidiToFreq},
};

// LYN: Public Interface

impl MidiToFreq {
    #[inline(always)]
    pub fn pin_input(
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::INPUT_MIDI => input_midi(pin, ui, snarl),
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
            Self::OUTPUT_FREQ => output_freq(pin, ui, snarl),
            _ => unreachable!("number node only has {} outputs", Self::OUTPUTS),
        }
    }
}

// LYN: Private Impl

type This = MidiToFreq;

fn input_midi(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<PatchNode>) -> PinInfo {
    let PatchNode::MidiToFreq(mtf) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(emath::vec2(60., 0.), egui::DragValue::new(&mut mtf.midi));
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_MIDI],
        This::INPUT_ACCEPT_MULTI[This::INPUT_MIDI],
    )
}

fn output_freq(
    pin: &egui_snarl::OutPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    let PatchNode::MidiToFreq(mtf) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label(format!("{:.2}", mtf.freq()));

    output_pin(This::OUTPUT_TYPE[This::OUTPUT_FREQ])
}

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
            oscillator::{Oscillator, Waveform},
        },
    },
};

// LYN: Public Interface

impl Oscillator {
    #[inline(always)]
    pub fn pin_input(
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::INPUT_FREQ => input_freq(pin, ui, snarl),
            Self::INPUT_PHASE => input_phase(pin, ui, snarl),
            Self::INPUT_WAVEFORM => input_waveform(pin, ui, snarl),
            Self::INPUT_RESET => input_reset(pin, ui, snarl),
            _ => unreachable!("oscillator only has {} inputs", Self::INPUTS),
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
            Self::OUTPUT_BLOCK => output_wave(pin, ui, snarl),
            _ => unreachable!("oscillator only has {} outputs", Self::OUTPUTS),
        }
    }
}

// LYN: Private Impl

type This = Oscillator;

#[inline(always)]
fn input_freq(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<NodeType>) -> PinInfo {
    let PatchNode::Oscillator(osc) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    let mut drag_value = egui::DragValue::new(&mut osc.freq_or_seed).range(Oscillator::FREQ_RANGE);
    if osc.waveform == Waveform::Noise {
        ui.label("种子");
    } else {
        ui.label("频率");
        drag_value = drag_value.suffix(" Hz");
    };

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(emath::vec2(60., 0.), drag_value);
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_FREQ],
        This::INPUT_ACCEPT_MULTI[This::INPUT_FREQ],
    )
}

#[inline(always)]
fn input_phase(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<NodeType>) -> PinInfo {
    let PatchNode::Oscillator(osc) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    if osc.waveform == Waveform::Noise {
        ui.disable();
    }

    ui.label("相位");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(60., 0.),
            egui::DragValue::new(&mut osc.phase)
                .range(Oscillator::PHASE_RANGE)
                .speed(0.01),
        );
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_PHASE],
        This::INPUT_ACCEPT_MULTI[This::INPUT_FREQ],
    )
}

#[inline(always)]
fn input_waveform(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
) -> PinInfo {
    let PatchNode::Oscillator(osc) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("波形");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        egui::ComboBox::new(
            WidgetId::SnarlNodeOscillatorWaveformComboBox(pin.id.node.0),
            "",
        )
        .width(60.)
        .selected_text(osc.waveform.name())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut osc.waveform, Waveform::Sine, "正弦 󰥛 ");
            ui.selectable_value(&mut osc.waveform, Waveform::Triangle, "三角 󱑼 ");
            ui.selectable_value(&mut osc.waveform, Waveform::Saw, "锯齿 󱑺 ");
            ui.selectable_value(&mut osc.waveform, Waveform::Square, "方波 󱑻 ");
            ui.selectable_value(&mut osc.waveform, Waveform::Noise, "噪声 󱩅 ");
        });
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_PHASE],
        This::INPUT_ACCEPT_MULTI[This::INPUT_FREQ],
    )
}

#[inline(always)]
fn input_reset(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<NodeType>) -> PinInfo {
    let PatchNode::Oscillator(osc) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("重置");

    match &*pin.remotes {
        // no input
        [] => {
            if ui.button("  ").clicked() {
                osc.reset();
            }
        }
        // TODO: one input
        [remote] => todo!(),
        _ => unreachable!("This phase input pin have at most 1 input"),
    };

    input_pin(
        This::INPUT_TYPE[This::INPUT_RESET],
        This::INPUT_ACCEPT_MULTI[This::INPUT_RESET],
    )
}

#[inline(always)]
fn output_wave(pin: &egui_snarl::OutPin, ui: &mut egui::Ui, snarl: &Snarl<NodeType>) -> PinInfo {
    output_pin(This::OUTPUT_TYPE[This::OUTPUT_BLOCK])
}

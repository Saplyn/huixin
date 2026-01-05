use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::{
        helpers::WidgetId,
        patch_viewer::{
            NodeType,
            constants::{input_pin, output_pin},
        },
    },
    model::patch::node::{
        PatchNode,
        oscillator::{Oscillator, Waveform},
    },
};

// LYN: Public Interface

#[inline(always)]
pub fn osc_input(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
    index: usize,
) -> PinInfo {
    match index {
        Oscillator::INPUT_FREQ => osc_input_freq(pin, ui, snarl),
        Oscillator::INPUT_PHASE => osc_input_phase(pin, ui, snarl),
        Oscillator::INPUT_WAVEFORM => osc_input_waveform(pin, ui, snarl),
        Oscillator::INPUT_RESET => osc_input_reset(pin, ui, snarl),
        _ => unreachable!("oscillator only has {} inputs", Oscillator::INPUTS),
    }
}

#[inline(always)]
pub fn osc_output(
    pin: &egui_snarl::OutPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
    index: usize,
) -> PinInfo {
    match index {
        Oscillator::OUTPUT_BLOCK => osc_output_wave(pin, ui, snarl),
        _ => unreachable!("oscillator only has {} outputs", Oscillator::OUTPUTS),
    }
}

// LYN: Private Impl

#[inline(always)]
fn osc_input_freq(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
) -> PinInfo {
    let PatchNode::Oscillator(osc) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    let drag_value = if osc.waveform == Waveform::Noise {
        ui.label("种子");
        egui::DragValue::new(&mut osc.freq_or_seed)
    } else {
        ui.label("频率");
        egui::DragValue::new(&mut osc.freq_or_seed).suffix(" Hz")
    };

    match &*pin.remotes {
        // no input
        [] => {
            ui.add_sized(emath::vec2(60., 0.), drag_value);
        }
        // TODO: one input
        [remote] => todo!(),
        _ => unreachable!("Oscillator freq input pin have at most 1 input"),
    }

    input_pin(
        Oscillator::INPUT_TYPE[Oscillator::INPUT_FREQ],
        Oscillator::INPUT_ACCEPT_MULTI[Oscillator::INPUT_FREQ],
    )
}

#[inline(always)]
fn osc_input_phase(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
) -> PinInfo {
    let PatchNode::Oscillator(osc) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    if osc.waveform == Waveform::Noise {
        ui.disable();
    }

    ui.label("相位");

    match &*pin.remotes {
        // no input
        [] => {
            ui.add_sized(
                emath::vec2(60., 0.),
                egui::DragValue::new(&mut osc.phase)
                    .range(0..=1)
                    .speed(0.01),
            );
        }
        // TODO: one input
        [remote] => todo!(),
        _ => unreachable!("Oscillator phase input pin have at most 1 input"),
    };

    input_pin(
        Oscillator::INPUT_TYPE[Oscillator::INPUT_PHASE],
        Oscillator::INPUT_ACCEPT_MULTI[Oscillator::INPUT_FREQ],
    )
}

#[inline(always)]
fn osc_input_waveform(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
) -> PinInfo {
    let PatchNode::Oscillator(osc) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("波形");

    match &*pin.remotes {
        // no input
        [] => {
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
        }
        // TODO: one input
        [remote] => todo!(),
        _ => unreachable!("Oscillator phase input pin have at most 1 input"),
    };

    input_pin(
        Oscillator::INPUT_TYPE[Oscillator::INPUT_PHASE],
        Oscillator::INPUT_ACCEPT_MULTI[Oscillator::INPUT_FREQ],
    )
}

#[inline(always)]
fn osc_input_reset(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<NodeType>,
) -> PinInfo {
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
        _ => unreachable!("Oscillator phase input pin have at most 1 input"),
    };

    input_pin(
        Oscillator::INPUT_TYPE[Oscillator::INPUT_RESET],
        Oscillator::INPUT_ACCEPT_MULTI[Oscillator::INPUT_RESET],
    )
}

#[inline(always)]
fn osc_output_wave(
    pin: &egui_snarl::OutPin,
    ui: &mut egui::Ui,
    snarl: &Snarl<NodeType>,
) -> PinInfo {
    output_pin(Oscillator::OUTPUT_TYPE[Oscillator::OUTPUT_BLOCK])
}

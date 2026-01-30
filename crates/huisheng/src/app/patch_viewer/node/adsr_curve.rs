use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::patch_viewer::{
        NodeType,
        constants::{input_pin, output_pin},
    },
    model::patch::node::{PatchNode, adsr_curve::ADSRCurve},
};

// LYN: Public Interface

impl ADSRCurve {
    #[inline(always)]
    pub fn pin_input(
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::INPUT_ATTACK => input_attack(pin, ui, snarl),
            Self::INPUT_DECAY => input_decay(pin, ui, snarl),
            Self::INPUT_SUSTAIN => input_sustain(pin, ui, snarl),
            Self::INPUT_RELEASE => input_release(pin, ui, snarl),
            Self::INPUT_PEAK => input_peak(pin, ui, snarl),
            Self::INPUT_KEEP => input_keep(pin, ui, snarl),
            Self::INPUT_TRIGGER => input_trigger(pin, ui, snarl),
            _ => unreachable!("adsr curve only has {} inputs", Self::INPUTS),
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
            Self::OUTPUT_BLOCK => output_block(pin, ui, snarl),
            _ => unreachable!("adsr curve only has {} outputs", Self::OUTPUTS),
        }
    }
}

// LYN: Private Impl

type This = ADSRCurve;

#[inline(always)]
fn input_attack(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    let PatchNode::ADSRCurve(adsr) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("攻击时长");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(60., 0.),
            egui::DragValue::new(&mut adsr.attack).range(0.0..=f64::MAX),
        );
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_ATTACK],
        This::INPUT_ACCEPT_MULTI[This::INPUT_ATTACK],
    )
}

#[inline(always)]
fn input_decay(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    let PatchNode::ADSRCurve(adsr) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("衰减时长");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(60., 0.),
            egui::DragValue::new(&mut adsr.decay).range(0.0..=f64::MAX),
        );
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_DECAY],
        This::INPUT_ACCEPT_MULTI[This::INPUT_DECAY],
    )
}

#[inline(always)]
fn input_sustain(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    let PatchNode::ADSRCurve(adsr) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("保持时长");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(60., 0.),
            egui::DragValue::new(&mut adsr.sustain).range(0.0..=f64::MAX),
        );
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_SUSTAIN],
        This::INPUT_ACCEPT_MULTI[This::INPUT_SUSTAIN],
    )
}

#[inline(always)]
fn input_release(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    let PatchNode::ADSRCurve(adsr) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("释放时长");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(60., 0.),
            egui::DragValue::new(&mut adsr.release).range(0.0..=f64::MAX),
        );
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_RELEASE],
        This::INPUT_ACCEPT_MULTI[This::INPUT_RELEASE],
    )
}

#[inline(always)]
fn input_peak(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<PatchNode>) -> PinInfo {
    let PatchNode::ADSRCurve(adsr) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("最高值");

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(60., 0.),
            egui::DragValue::new(&mut adsr.peak).speed(0.1),
        );
    });

    input_pin(
        This::INPUT_TYPE[This::INPUT_PEAK],
        This::INPUT_ACCEPT_MULTI[This::INPUT_PEAK],
    )
}

#[inline(always)]
fn input_keep(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<PatchNode>) -> PinInfo {
    let PatchNode::ADSRCurve(adsr) = &mut snarl[pin.id.node] else {
        unreachable!();
    };
    ui.label("保持值");
    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(60., 0.),
            egui::DragValue::new(&mut adsr.keep).speed(0.1),
        );
    });
    input_pin(
        This::INPUT_TYPE[This::INPUT_KEEP],
        This::INPUT_ACCEPT_MULTI[This::INPUT_KEEP],
    )
}

#[inline(always)]
fn input_trigger(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    let PatchNode::ADSRCurve(adsr) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.label("触发");

    if ui.button("  ").clicked() {
        adsr.trigger();
    }

    input_pin(
        This::INPUT_TYPE[This::INPUT_TRIGGER],
        This::INPUT_ACCEPT_MULTI[This::INPUT_TRIGGER],
    )
}

#[inline(always)]
fn output_block(
    pin: &egui_snarl::OutPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    output_pin(This::OUTPUT_TYPE[This::OUTPUT_BLOCK])
}

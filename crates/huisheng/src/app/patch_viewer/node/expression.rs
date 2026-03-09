use egui_snarl::{Snarl, ui::PinInfo};

use crate::{
    app::patch_viewer::{
        NodeTy,
        constants::{input_pin, output_pin},
    },
    model::patch::{
        WireDataType,
        node::{PatchNode, PatchNodeTrait, adsr_curve::ADSRCurve, expression::Expression},
    },
};

// LYN: Public Interface

impl Expression {
    #[inline(always)]
    pub fn pin_input(
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeTy>,
        index: usize,
    ) -> PinInfo {
        let PatchNode::Expression(expr) = &snarl[pin.id.node] else {
            unreachable!();
        };
        let inputs = expr.inputs();

        match index {
            Self::INPUT_EXPR => input_expr(pin, ui, snarl),
            ind if ind < inputs => input_binding(pin, ui, snarl, ind),
            _ => unreachable!("this expression only has {} inputs", inputs),
        }
    }

    #[inline(always)]
    pub fn pin_output(
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeTy>,
        index: usize,
    ) -> PinInfo {
        match index {
            Self::OUTPUT_RESULT => output_result(pin, ui, snarl),
            _ => unreachable!("expression only has {} outputs", Self::OUTPUTS),
        }
    }
}

// LYN: Private Impl

type This = Expression;

fn input_expr(pin: &egui_snarl::InPin, ui: &mut egui::Ui, snarl: &mut Snarl<PatchNode>) -> PinInfo {
    let PatchNode::Expression(expr) = &mut snarl[pin.id.node] else {
        unreachable!();
    };

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(80., 0.),
            egui::TextEdit::singleline(&mut expr.expr),
        );
    });

    input_pin(This::INPUT_TYPE[This::INPUT_EXPR], false)
}

fn input_binding(
    pin: &egui_snarl::InPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
    index: usize,
) -> PinInfo {
    let PatchNode::Expression(expr) = &mut snarl[pin.id.node] else {
        unreachable!();
    };
    let mapping_index = index - This::FIXED_INPUTS;

    ui.label(&expr.bindings[mapping_index].0);

    ui.add_enabled_ui(pin.remotes.is_empty(), |ui| {
        ui.add_sized(
            emath::vec2(60., 0.),
            egui::DragValue::new(&mut expr.bindings[mapping_index].1),
        );
    });

    input_pin(WireDataType::Number, false)
}

fn output_result(
    pin: &egui_snarl::OutPin,
    ui: &mut egui::Ui,
    snarl: &mut Snarl<PatchNode>,
) -> PinInfo {
    output_pin(This::OUTPUT_TYPE[This::OUTPUT_RESULT])
}

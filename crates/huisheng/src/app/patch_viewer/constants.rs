use egui_snarl::ui::{PinInfo, PinShape};

use crate::model::patch::PatchOutputType;

pub const INPUT_PIN_SINGLE: PinInfo = default_pin().with_shape(PinShape::Circle);
pub const INPUT_PIN_MULTIPLE: PinInfo = default_pin().with_shape(PinShape::Square);
pub const OUTPUT_PIN: PinInfo = default_pin().with_shape(PinShape::Circle);

pub const PIN_COLOR_BLOCK: ecolor::Color32 = ecolor::Color32::RED;
pub const PIN_COLOR_NUMBER: ecolor::Color32 = ecolor::Color32::BLUE;
pub const PIN_COLOR_TEXT: ecolor::Color32 = ecolor::Color32::GREEN;
pub const PIN_COLOR_BANG: ecolor::Color32 = ecolor::Color32::YELLOW;

const fn default_pin() -> PinInfo {
    PinInfo {
        shape: None,
        fill: None,
        stroke: None,
        wire_color: None,
        wire_style: None,
        position: None,
    }
}

pub const fn input_pin(pin_type: PatchOutputType, multi: bool) -> PinInfo {
    let pin = match multi {
        true => INPUT_PIN_MULTIPLE,
        false => INPUT_PIN_SINGLE,
    };
    match pin_type {
        PatchOutputType::Number => pin.with_fill(PIN_COLOR_NUMBER),
        PatchOutputType::Text => pin.with_fill(PIN_COLOR_TEXT),
        PatchOutputType::Block => pin.with_fill(PIN_COLOR_BLOCK),
        PatchOutputType::Bang => pin.with_fill(PIN_COLOR_BANG),
    }
}

pub const fn output_pin(pin_type: PatchOutputType) -> PinInfo {
    match pin_type {
        PatchOutputType::Number => OUTPUT_PIN.with_fill(PIN_COLOR_NUMBER),
        PatchOutputType::Text => OUTPUT_PIN.with_fill(PIN_COLOR_TEXT),
        PatchOutputType::Block => OUTPUT_PIN.with_fill(PIN_COLOR_BLOCK),
        PatchOutputType::Bang => OUTPUT_PIN.with_fill(PIN_COLOR_BANG),
    }
}

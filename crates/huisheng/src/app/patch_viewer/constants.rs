use egui_snarl::ui::{PinInfo, PinShape};

use crate::model::patch::WireDataType;

pub const INPUT_PIN_SINGLE: PinInfo = default_pin().with_shape(PinShape::Circle);
pub const INPUT_PIN_MULTIPLE: PinInfo = default_pin().with_shape(PinShape::Square);
pub const OUTPUT_PIN: PinInfo = default_pin().with_shape(PinShape::Circle);

pub const PIN_COLOR_BLOCK: ecolor::Color32 = ecolor::Color32::RED;
pub const PIN_COLOR_NUMBER: ecolor::Color32 = ecolor::Color32::BLUE;
pub const PIN_COLOR_TEXT: ecolor::Color32 = ecolor::Color32::GREEN;
pub const PIN_COLOR_BANG: ecolor::Color32 = ecolor::Color32::YELLOW;
pub const PIN_COLOR_NONBLOCK: ecolor::Color32 = ecolor::Color32::LIGHT_GRAY;
pub const PIN_COLOR_STATIC_NONBLOCK: ecolor::Color32 = ecolor::Color32::DARK_GRAY;

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

pub const fn input_pin(pin_type: WireDataType, multi: bool) -> PinInfo {
    let pin = match multi {
        true => INPUT_PIN_MULTIPLE,
        false => INPUT_PIN_SINGLE,
    };
    match pin_type {
        WireDataType::Number => pin.with_fill(PIN_COLOR_NUMBER),
        WireDataType::Text => pin.with_fill(PIN_COLOR_TEXT),
        WireDataType::Block => pin.with_fill(PIN_COLOR_BLOCK),
        WireDataType::Bang => pin.with_fill(PIN_COLOR_BANG),
        WireDataType::NonBlock => pin.with_fill(PIN_COLOR_NONBLOCK),
        WireDataType::Constant => pin.with_fill(PIN_COLOR_STATIC_NONBLOCK),
    }
}

pub const fn output_pin(pin_type: WireDataType) -> PinInfo {
    match pin_type {
        WireDataType::Number => OUTPUT_PIN.with_fill(PIN_COLOR_NUMBER),
        WireDataType::Text => OUTPUT_PIN.with_fill(PIN_COLOR_TEXT),
        WireDataType::Block => OUTPUT_PIN.with_fill(PIN_COLOR_BLOCK),
        WireDataType::Bang => OUTPUT_PIN.with_fill(PIN_COLOR_BANG),
        WireDataType::NonBlock => OUTPUT_PIN.with_fill(PIN_COLOR_NONBLOCK),
        WireDataType::Constant => OUTPUT_PIN.with_fill(PIN_COLOR_STATIC_NONBLOCK),
    }
}

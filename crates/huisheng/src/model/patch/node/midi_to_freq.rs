use egui_snarl::{NodeId, OutPinId};
use serde::{Deserialize, Serialize};

use crate::model::{
    data_mem::NonBlockData,
    patch::{Number, WireDataType, node::PatchNodeTrait},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct MidiToFreq {
    pub midi: u8,

    input_id: Option<OutPinId>,
}

impl MidiToFreq {
    pub const NAME: &str = "MIDI 转频率";
    pub const INPUTS: usize = 1;
    pub const OUTPUTS: usize = 1;

    pub const INPUT_MIDI: usize = 0;
    pub const INPUT_TYPE: [WireDataType; Self::INPUTS] = [WireDataType::Number];
    pub const INPUT_ACCEPT_MULTI: [bool; Self::INPUTS] = [false];

    pub const OUTPUT_FREQ: usize = 0;
    pub const OUTPUT_TYPE: [WireDataType; Self::OUTPUTS] = [WireDataType::Number];
}

impl MidiToFreq {
    pub fn new() -> Self {
        Self {
            midi: 0,
            input_id: None,
        }
    }
}

impl PatchNodeTrait for MidiToFreq {
    fn name(&self) -> &str {
        Self::NAME
    }
    fn inputs(&self) -> usize {
        Self::INPUTS
    }
    fn outputs(&self) -> usize {
        Self::OUTPUTS
    }
    fn pin_accept_multi(&self, pin_index: usize) -> bool {
        Self::INPUT_ACCEPT_MULTI[pin_index]
    }
    fn input_type(&self, pin_index: usize) -> WireDataType {
        Self::INPUT_TYPE[pin_index]
    }
    fn output_type(&self, pin_index: usize) -> WireDataType {
        Self::OUTPUT_TYPE[pin_index]
    }
    fn input_for_pin(&self, pin_index: usize) -> Option<OutPinId> {
        assert_eq!(pin_index, Self::INPUT_MIDI);
        self.input_id
    }
    fn take_input(&mut self, _pin_index: usize, source: OutPinId) {
        self.input_id = Some(source);
    }
    fn drop_input(&mut self, _pin_index: usize, _source: OutPinId) {
        self.input_id = None;
    }
    fn output_number(&self, pin_index: usize) -> Option<Number> {
        assert_eq!(pin_index, Self::OUTPUT_FREQ);
        Some(self.freq())
    }
    fn output_arbitrary(&mut self, pin_index: usize, _node_id: NodeId) -> Option<NonBlockData> {
        assert_eq!(pin_index, Self::OUTPUT_FREQ);
        self.output_number(pin_index).map(NonBlockData::Number)
    }
}

impl MidiToFreq {
    pub fn freq(&self) -> Number {
        440. * 2f64.powf((self.midi as Number - 69.) / 12.)
    }
}

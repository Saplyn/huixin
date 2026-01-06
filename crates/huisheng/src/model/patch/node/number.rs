use egui_snarl::OutPinId;
use serde::{Deserialize, Serialize};

use crate::model::patch::{Number, PatchOutputType, node::PatchNodeTrait};

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberNode {
    pub number: Number,

    #[serde(skip)]
    input_id: Option<OutPinId>,
}

impl NumberNode {
    pub const INPUTS: usize = 1;
    pub const OUTPUTS: usize = 1;

    pub const INPUT_NUM: usize = 0;
    pub const INPUT_TYPE: [PatchOutputType; Self::INPUTS] = [PatchOutputType::Number];
    pub const INPUT_ACCEPT_MULTI: [bool; Self::INPUTS] = [false];

    pub const OUTPUT_NUM: usize = 0;
    pub const OUTPUT_TYPE: [PatchOutputType; Self::OUTPUTS] = [PatchOutputType::Number];
}

impl NumberNode {
    pub fn new() -> Self {
        Self {
            number: 0.,
            input_id: None,
        }
    }
}

impl PatchNodeTrait for NumberNode {
    fn name(&self) -> &str {
        "数字"
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
    fn input_type(&self, pin_index: usize) -> PatchOutputType {
        Self::INPUT_TYPE[pin_index]
    }
    fn output_type(&self, pin_index: usize) -> PatchOutputType {
        Self::OUTPUT_TYPE[pin_index]
    }
    fn take_input(&mut self, _pin_index: usize, source: OutPinId) {
        self.input_id = Some(source);
    }
    fn drop_input(&mut self, _pin_index: usize, _source: OutPinId) {
        self.input_id = None;
    }
    fn output_number(&self, pin_index: usize) -> Option<Number> {
        assert_eq!(pin_index, Self::OUTPUT_NUM);
        Some(self.number)
    }
}

impl NumberNode {
    pub fn input_for_pin(&self, pin_index: usize) -> &Option<OutPinId> {
        assert_eq!(pin_index, Self::INPUT_NUM);
        &self.input_id
    }
}

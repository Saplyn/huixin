use std::collections::HashSet;

use egui_snarl::OutPinId;
use serde::{Deserialize, Serialize};

use crate::model::patch::{PatchOutputType, node::PatchNodeTrait};

#[derive(Debug, Serialize, Deserialize)]
pub struct Speaker {
    #[serde(skip)]
    input_ids: [HashSet<OutPinId>; Self::INPUTS],
}

impl Speaker {
    pub const INPUTS: usize = 2;
    pub const OUTPUTS: usize = 0;

    pub const INPUT_LEFT_CHAN: usize = 0;
    pub const INPUT_RIGHT_CHAN: usize = 1;
    pub const INPUT_TYPE: [PatchOutputType; Self::INPUTS] =
        [PatchOutputType::Block, PatchOutputType::Block];
    pub const INPUT_ACCEPT_MULTI: [bool; Self::INPUTS] = [true, true];

    pub const OUTPUT_TYPE: [PatchOutputType; Self::OUTPUTS] = [];
}

impl PatchNodeTrait for Speaker {
    fn name(&self) -> &str {
        "扬声器"
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
    fn take_input(&mut self, pin_index: usize, source: OutPinId) {
        self.input_ids[pin_index].insert(source);
    }
    fn drop_input(&mut self, pin_index: usize, source: OutPinId) {
        self.input_ids[pin_index].remove(&source);
    }
}

impl Speaker {
    pub fn new() -> Self {
        Self {
            input_ids: [HashSet::new(), HashSet::new()],
        }
    }
    pub fn inputs_for_pin(&self, pin_index: usize) -> &HashSet<OutPinId> {
        &self.input_ids[pin_index]
    }
}

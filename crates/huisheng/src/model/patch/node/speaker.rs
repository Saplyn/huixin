use std::collections::HashSet;

use egui_snarl::{NodeId, OutPinId};

use crate::model::patch::PatchOutputType;

#[derive(Debug)]
pub struct Speaker {
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

impl Speaker {
    pub fn new() -> Self {
        Self {
            input_ids: [HashSet::new(), HashSet::new()],
        }
    }
    pub fn inputs_for_pin(&self, pin_index: usize) -> &HashSet<OutPinId> {
        &self.input_ids[pin_index]
    }
    pub fn add_input(&mut self, pin_index: usize, source: OutPinId) {
        self.input_ids[pin_index].insert(source);
    }
    pub fn del_input(&mut self, pin_index: usize, source: OutPinId) {
        self.input_ids[pin_index].remove(&source);
    }
}

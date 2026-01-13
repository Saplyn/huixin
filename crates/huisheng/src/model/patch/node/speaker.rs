use std::collections::HashSet;

use egui_snarl::{NodeId, OutPinId};
use serde::{Deserialize, Serialize};

use crate::model::{
    data_mem::NonBlockData,
    patch::{WireDataType, node::PatchNodeTrait},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Speaker {
    input_ids: [HashSet<OutPinId>; Self::INPUTS],
}

impl Speaker {
    pub const NAME: &str = "扬声器";
    pub const INPUTS: usize = 2;
    pub const OUTPUTS: usize = 0;

    pub const INPUT_LEFT_CHAN: usize = 0;
    pub const INPUT_RIGHT_CHAN: usize = 1;
    pub const INPUT_TYPE: [WireDataType; Self::INPUTS] = [WireDataType::Block, WireDataType::Block];
    pub const INPUT_ACCEPT_MULTI: [bool; Self::INPUTS] = [true, true];

    pub const OUTPUT_TYPE: [WireDataType; Self::OUTPUTS] = [];
}

impl PatchNodeTrait for Speaker {
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
    fn inputs_for_pin(&self, pin_index: usize) -> Option<&HashSet<OutPinId>> {
        Some(&self.input_ids[pin_index])
    }
    fn take_input(&mut self, pin_index: usize, source: OutPinId) {
        self.input_ids[pin_index].insert(source);
    }
    fn drop_input(&mut self, pin_index: usize, source: OutPinId) {
        self.input_ids[pin_index].remove(&source);
    }
    fn output_arbitrary(&mut self, _pin_index: usize, _node_id: NodeId) -> Option<NonBlockData> {
        unreachable!("Speaker has {} output pins", Self::OUTPUTS);
    }
}

impl Speaker {
    pub fn new() -> Self {
        Self {
            input_ids: [HashSet::new(), HashSet::new()],
        }
    }
}

use std::collections::HashMap;

use egui_snarl::{InPinId, NodeId, OutPinId};
use serde::{Deserialize, Serialize};

use crate::model::{
    data_mem::NonBlockData,
    patch::{Bang, WireDataType, node::PatchNodeTrait},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BangNode {
    pub bang: Bang,
    pub data: NonBlockData,

    mem: HashMap<NodeId, NonBlockData>,
    input_id: Option<OutPinId>,
}

impl BangNode {
    pub const NAME: &str = "触发器";
    pub const INPUTS: usize = 1;
    pub const OUTPUTS: usize = 1;

    pub const INPUT_ARBITRARY: usize = 0;
    pub const INPUT_TYPE: [WireDataType; Self::INPUTS] = [WireDataType::NonBlock];
    pub const INPUT_ACCEPT_MULTI: [bool; Self::INPUTS] = [false];

    pub const OUTPUT_BANG: usize = 0;
    pub const OUTPUT_TYPE: [WireDataType; Self::OUTPUTS] = [WireDataType::Bang];
}

impl BangNode {
    pub fn new() -> Self {
        Self {
            bang: false,
            data: NonBlockData::Bang(false),
            mem: HashMap::new(),
            input_id: None,
        }
    }
}

impl PatchNodeTrait for BangNode {
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
        assert_eq!(pin_index, Self::INPUT_ARBITRARY);
        self.input_id
    }
    fn take_input(&mut self, _pin_index: usize, source: OutPinId) {
        self.input_id = Some(source);
    }
    fn drop_input(&mut self, _pin_index: usize, _source: OutPinId) {
        self.input_id = None;
    }
    fn output_bang(&mut self, pin_index: usize, node_id: NodeId) -> Option<Bang> {
        assert_eq!(pin_index, Self::OUTPUT_BANG);

        if self.bang {
            self.bang = false;
            return Some(true);
        }

        let bang = if let Some(mem) = self.mem.get_mut(&node_id) {
            if self.data != *mem {
                *mem = self.data.clone();
                true
            } else {
                false
            }
        } else {
            self.mem.insert(node_id, self.data.clone());
            true
        };

        Some(bang)
    }
    fn output_arbitrary(&mut self, pin_index: usize, node_id: NodeId) -> Option<NonBlockData> {
        assert_eq!(pin_index, Self::OUTPUT_BANG);
        self.output_bang(pin_index, node_id).map(NonBlockData::Bang)
    }
    fn post_process(&mut self) {
        self.bang = false;
    }
    fn on_output_connect(&mut self, pin_index: usize, remote: InPinId) {
        assert_eq!(pin_index, Self::OUTPUT_BANG);
        self.mem.insert(remote.node, self.data.clone());
    }
    fn on_output_disconnect(&mut self, pin_index: usize, remote: InPinId) {
        assert_eq!(pin_index, Self::OUTPUT_BANG);
        self.mem.remove(&remote.node);
    }
}

impl BangNode {
    pub fn update(&mut self, hint: Option<NonBlockData>) {
        if let Some(data) = hint {
            self.data = data;
        } else {
            self.bang = true;
        }
    }
}

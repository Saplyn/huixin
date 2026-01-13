use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use egui_snarl::{InPinId, NodeId, OutPinId};
use serde::{Deserialize, Serialize};

use crate::model::{
    data_mem::{MemData, NonBlockData},
    patch::{Bang, WireDataType, node::PatchNodeTrait},
    state::CentralState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteData {
    pub tag: String,
    pub prop: String,

    input_ids: [Option<OutPinId>; Self::INPUTS],

    #[serde(skip)]
    mem: HashMap<NodeId, MemData>,
    #[serde(skip)]
    state: OnceLock<Arc<CentralState>>,
}

impl RemoteData {
    pub const NAME: &str = "网络数据";
    pub const INPUTS: usize = 2;
    pub const OUTPUTS: usize = 2;

    pub const INPUT_TAG: usize = 0;
    pub const INPUT_PROP: usize = 1;
    pub const INPUT_TYPE: [WireDataType; Self::INPUTS] = [WireDataType::Text, WireDataType::Text];
    pub const INPUT_ACCEPT_MULTI: [bool; Self::INPUTS] = [false, false];

    pub const OUTPUT_DATA: usize = 0;
    pub const OUTPUT_BANG: usize = 1;
    pub const OUTPUT_TYPE: [WireDataType; Self::OUTPUTS] =
        [WireDataType::Constant, WireDataType::Bang];
}

impl RemoteData {
    pub fn new() -> Self {
        Self {
            tag: String::new(),
            prop: String::new(),
            input_ids: [None; Self::OUTPUTS],
            mem: HashMap::new(),
            state: OnceLock::new(),
        }
    }
}

impl PatchNodeTrait for RemoteData {
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
        self.input_ids[pin_index]
    }
    fn take_input(&mut self, pin_index: usize, source: OutPinId) {
        self.input_ids[pin_index] = Some(source);
    }
    fn drop_input(&mut self, pin_index: usize, _source: OutPinId) {
        self.input_ids[pin_index] = None;
    }
    fn output_number(&self, pin_index: usize) -> Option<crate::model::patch::Number> {
        if pin_index == Self::OUTPUT_BANG {
            return None;
        }
        assert_eq!(pin_index, Self::OUTPUT_DATA);
        if let Some(NonBlockData::Number(num)) = self.data().as_ref().map(|d| &d.inner) {
            Some(*num)
        } else {
            None
        }
    }
    fn output_text(&self, pin_index: usize) -> Option<String> {
        if pin_index == Self::OUTPUT_BANG {
            return None;
        }
        assert_eq!(pin_index, Self::OUTPUT_DATA);
        if let Some(NonBlockData::Text(text)) = self.data().as_ref().map(|d| &d.inner) {
            Some(text.clone())
        } else {
            None
        }
    }
    fn output_bang(&mut self, pin_index: usize, node_id: NodeId) -> Option<Bang> {
        if pin_index != Self::OUTPUT_BANG {
            return None;
        }
        let data = self.data()?;
        let bang = if let Some(mem) = self.mem.get_mut(&node_id) {
            if *mem != data {
                *mem = data;
                true
            } else {
                false
            }
        } else {
            self.mem.insert(node_id, data);
            true
        };
        Some(bang)
    }
    fn output_arbitrary(&mut self, pin_index: usize, node_id: NodeId) -> Option<NonBlockData> {
        if pin_index == Self::OUTPUT_BANG {
            return self.output_bang(pin_index, node_id).map(NonBlockData::Bang);
        }
        assert_eq!(pin_index, Self::OUTPUT_DATA);
        self.data().map(|mem| mem.inner)
    }
    fn pre_process(&mut self, state: Arc<CentralState>) {
        let _ = self.state.set(state);
    }
    fn on_output_connect(&mut self, pin_index: usize, remote: InPinId) {
        if pin_index != Self::OUTPUT_BANG {
            return;
        }
        if let Some(data) = self.data() {
            self.mem.insert(remote.node, data);
        }
    }
    fn on_output_disconnect(&mut self, pin_index: usize, remote: InPinId) {
        if pin_index != Self::OUTPUT_BANG {
            return;
        }
        self.mem.remove(&remote.node);
    }
}

impl RemoteData {
    fn data(&self) -> Option<MemData> {
        self.state
            .get()
            .and_then(|state| state.data_mem_get(&self.tag, &self.prop))
    }
}

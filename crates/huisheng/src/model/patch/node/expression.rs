use std::collections::BTreeSet;

use egui_snarl::{NodeId, OutPinId, Snarl};
use evalexpr::DefaultNumericTypes;
use serde::{Deserialize, Serialize};

use crate::{
    model::{
        data_mem::NonBlockData,
        patch::{
            Block, Number, WireDataType,
            node::{PatchNode, PatchNodeTrait},
        },
    },
    node_or_bail,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Expression {
    pub expr: String,

    input_ids: Vec<Option<OutPinId>>,

    #[serde(skip)]
    pub result: Number,

    pub last_expr: String,
    pub bindings: Vec<(String, Number)>,
    #[serde(skip)]
    pub op_tree: Option<evalexpr::Node<DefaultNumericTypes>>,
}

impl Expression {
    pub const NAME: &str = "表达式";
    pub const FIXED_INPUTS: usize = 1;
    pub const OUTPUTS: usize = 1;

    pub const INPUT_EXPR: usize = 0;
    pub const INPUT_TYPE: [WireDataType; Self::FIXED_INPUTS] = [WireDataType::Text];

    pub const OUTPUT_RESULT: usize = 0;
    pub const OUTPUT_TYPE: [WireDataType; Self::OUTPUTS] = [WireDataType::Number];
}

impl Expression {
    pub fn new() -> Self {
        Self {
            expr: String::new(),
            input_ids: Vec::new(),
            result: 0.0,
            last_expr: String::new(),
            op_tree: None,
            bindings: Vec::new(),
        }
    }
}

impl PatchNodeTrait for Expression {
    fn name(&self) -> &str {
        Self::NAME
    }
    fn inputs(&self) -> usize {
        Self::FIXED_INPUTS + self.bindings.len()
    }
    fn outputs(&self) -> usize {
        Self::OUTPUTS
    }
    fn pin_accept_multi(&self, _pin_index: usize) -> bool {
        false
    }
    fn input_type(&self, pin_index: usize) -> WireDataType {
        if pin_index < Self::FIXED_INPUTS {
            Self::INPUT_TYPE[pin_index]
        } else {
            WireDataType::Number
        }
    }
    fn output_type(&self, pin_index: usize) -> WireDataType {
        Self::OUTPUT_TYPE[pin_index]
    }
    fn input_for_pin(&self, pin_index: usize) -> Option<OutPinId> {
        self.input_ids.get(pin_index).copied().flatten()
    }
    fn take_input(&mut self, pin_index: usize, source: OutPinId) {
        self.input_ids[pin_index] = Some(source);
    }
    fn drop_input(&mut self, pin_index: usize, _source: OutPinId) {
        self.input_ids[pin_index] = None;
    }
    fn output_number(&self, pin_index: usize) -> Option<Number> {
        assert_eq!(pin_index, Self::OUTPUT_RESULT);
        Some(self.result)
    }
    fn output_nonblock(&mut self, pin_index: usize, _node_id: NodeId) -> Option<NonBlockData> {
        assert_eq!(pin_index, Self::OUTPUT_RESULT);
        self.output_number(pin_index).map(NonBlockData::Number)
    }
}

impl Expression {
    pub fn clear_all_connections_and_resize(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
    ) -> Result<(), ()> {
        let wires: Vec<_> = snarl.wires().filter(|(_, to)| to.node == node_id).collect();
        for (from, to) in wires {
            snarl.disconnect(from, to);
        }

        let PatchNode::Expression(expr) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };
        expr.input_ids.clear();
        expr.input_ids.resize(expr.inputs(), None);
        Ok(())
    }
    pub fn input_ids(&self) -> Vec<Option<OutPinId>> {
        self.input_ids.clone()
    }
}

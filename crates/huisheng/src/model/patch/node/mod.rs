use std::{collections::HashSet, sync::Arc};

use egui_snarl::{InPinId, NodeId, OutPinId};

use crate::model::{
    data_mem::NonBlockData,
    patch::{
        Bang, Block, Number, WireDataType,
        node::{
            bang::BangNode, number::NumberNode, oscillator::Oscillator, remote_data::RemoteData,
            speaker::Speaker,
        },
    },
    state::CentralState,
};

pub mod bang;
pub mod number;
pub mod oscillator;
pub mod remote_data;
pub mod speaker;

// LYN: Snarl Node Impl

#[derive(Debug)]
pub enum PatchNode {
    // Signal
    Oscillator(Box<Oscillator>),
    Speaker(Speaker),

    // Communication
    // RemoteData(RemoteData),

    // Logic

    // Variable
    Number(NumberNode),
    // Text(TextNode),
    Bang(BangNode),
    //

    // Calculation
    // Expression(Expression),
    // ADSRCurve(ADSRCurve),
    // MidiToFreq,

    // Processing
    // WaveAdder(WaveAdder),
    // WaveMultiplier(WaveMultiplier),
    // WaveOffseter(WaveOffseter),
    // WaveScaler(WaveScaler),
    // WaveClipper(WaveClipper),

    // Communication
    RemoteData(RemoteData),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchNodeType {
    // Signal
    Oscillator,
    Speaker,

    // Variable
    Number,
    // Text,
    Bang,
    //

    // Calculation
    // Expression,
    // ADSRCurve,
    // MidiToFreq,

    // Processing
    // WaveAdder,
    // WaveMultiplier,
    // WaveOffseter,
    // WaveScaler,
    // WaveClipper,

    // Communication
    RemoteData,
}

pub trait PatchNodeTrait {
    fn name(&self) -> &str;
    fn inputs(&self) -> usize;
    fn outputs(&self) -> usize;
    fn pin_accept_multi(&self, pin_index: usize) -> bool;
    fn input_type(&self, pin_index: usize) -> WireDataType;
    fn output_type(&self, pin_index: usize) -> WireDataType;
    fn take_input(&mut self, pin_index: usize, source: OutPinId);
    fn drop_input(&mut self, pin_index: usize, source: OutPinId);
    fn input_for_pin(&self, pin_index: usize) -> Option<OutPinId> {
        let _ = pin_index;
        None
    }
    fn inputs_for_pin(&self, pin_index: usize) -> Option<&HashSet<OutPinId>> {
        let _ = pin_index;
        None
    }
    fn output_block(&self, pin_index: usize) -> Option<&Block> {
        let _ = pin_index;
        None
    }
    fn output_number(&self, pin_index: usize) -> Option<Number> {
        let _ = pin_index;
        None
    }
    fn output_text(&self, pin_index: usize) -> Option<String> {
        let _ = pin_index;
        None
    }
    fn output_bang(&mut self, pin_index: usize, node_id: NodeId) -> Option<Bang> {
        let _ = (pin_index, node_id);
        None
    }
    fn output_arbitrary(&mut self, pin_index: usize, node_id: NodeId) -> Option<NonBlockData>;
    fn pre_process(&mut self, state: Arc<CentralState>) {
        let _ = state;
    }
    fn post_process(&mut self) {}
    fn on_output_connect(&mut self, pin_index: usize, remote: InPinId) {
        let _ = (pin_index, remote);
    }
    fn on_output_disconnect(&mut self, pin_index: usize, remote: InPinId) {
        let _ = (pin_index, remote);
    }
}

impl PatchNode {
    #[inline]
    pub fn get_type(&self) -> PatchNodeType {
        match self {
            // Signal
            PatchNode::Oscillator(_) => PatchNodeType::Oscillator,
            PatchNode::Speaker(_) => PatchNodeType::Speaker,

            // Variable
            PatchNode::Number(_) => PatchNodeType::Number,
            // PatchNode::Text(_) => PatchNodeType::Text,
            PatchNode::Bang(_) => PatchNodeType::Bang,
            //

            // Calculation
            // PatchNode::Expression(_) => PatchNodeType::Expression,
            // PatchNode::ADSRCurve(_) => PatchNodeType::ADSRCurve,
            // PatchNode::MidiToFreq => PatchNodeType::MidiToFreq,

            // Processing
            // PatchNode::WaveAdder(_) => PatchNodeType::WaveAdder,
            // PatchNode::WaveMultiplier(_) => PatchNodeType::WaveMultiplier,
            // PatchNode::WaveOffseter(_) => PatchNodeType::WaveOffseter,
            // PatchNode::WaveScaler(_) => PatchNodeType::WaveScaler,
            // PatchNode::WaveClipper(_) => PatchNodeType::WaveClipper,

            // Communication
            PatchNode::RemoteData(_) => PatchNodeType::RemoteData,
        }
    }
}

macro_rules! delegate_to_node {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            PatchNode::Oscillator(osc) => osc.$method($($arg),*),
            PatchNode::Speaker(speaker) => speaker.$method($($arg),*),
            PatchNode::Number(num) => num.$method($($arg),*),
            PatchNode::Bang(bang) => bang.$method($($arg),*),
            PatchNode::RemoteData(remote) => remote.$method($($arg),*),
        }
    };
}

impl PatchNodeTrait for PatchNode {
    #[inline]
    fn name(&self) -> &str {
        delegate_to_node!(self, name)
    }

    #[inline]
    fn inputs(&self) -> usize {
        delegate_to_node!(self, inputs)
    }

    #[inline]
    fn outputs(&self) -> usize {
        delegate_to_node!(self, outputs)
    }

    #[inline]
    fn pin_accept_multi(&self, pin_index: usize) -> bool {
        delegate_to_node!(self, pin_accept_multi, pin_index)
    }

    #[inline]
    fn input_type(&self, pin_index: usize) -> WireDataType {
        delegate_to_node!(self, input_type, pin_index)
    }

    #[inline]
    fn output_type(&self, pin_index: usize) -> WireDataType {
        delegate_to_node!(self, output_type, pin_index)
    }

    #[inline]
    fn input_for_pin(&self, pin_index: usize) -> Option<OutPinId> {
        assert!(!self.pin_accept_multi(pin_index));
        delegate_to_node!(self, input_for_pin, pin_index)
    }

    #[inline]
    fn inputs_for_pin(&self, pin_index: usize) -> Option<&HashSet<OutPinId>> {
        assert!(self.pin_accept_multi(pin_index));
        delegate_to_node!(self, inputs_for_pin, pin_index)
    }

    #[inline]
    fn take_input(&mut self, pin_index: usize, source: OutPinId) {
        delegate_to_node!(self, take_input, pin_index, source)
    }

    #[inline]
    fn drop_input(&mut self, pin_index: usize, source: OutPinId) {
        delegate_to_node!(self, drop_input, pin_index, source)
    }

    #[inline]
    fn output_block(&self, pin_index: usize) -> Option<&Block> {
        delegate_to_node!(self, output_block, pin_index)
    }

    #[inline]
    fn output_number(&self, pin_index: usize) -> Option<Number> {
        delegate_to_node!(self, output_number, pin_index)
    }

    #[inline]
    fn output_text(&self, pin_index: usize) -> Option<String> {
        delegate_to_node!(self, output_text, pin_index)
    }

    #[inline]
    fn output_bang(&mut self, pin_index: usize, node_id: NodeId) -> Option<Bang> {
        delegate_to_node!(self, output_bang, pin_index, node_id)
    }

    #[inline]
    fn output_arbitrary(&mut self, pin_index: usize, node_id: NodeId) -> Option<NonBlockData> {
        delegate_to_node!(self, output_arbitrary, pin_index, node_id)
    }

    #[inline]
    fn pre_process(&mut self, state: Arc<CentralState>) {
        delegate_to_node!(self, pre_process, state)
    }

    #[inline]
    fn post_process(&mut self) {
        delegate_to_node!(self, post_process)
    }

    #[inline]
    fn on_output_connect(&mut self, pin_index: usize, remote: InPinId) {
        delegate_to_node!(self, on_output_connect, pin_index, remote)
    }

    #[inline]
    fn on_output_disconnect(&mut self, pin_index: usize, remote: InPinId) {
        delegate_to_node!(self, on_output_disconnect, pin_index, remote)
    }
}

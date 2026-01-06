use egui_snarl::OutPinId;

use crate::model::patch::{
    Block, Number, PatchOutputType,
    node::{number::NumberNode, oscillator::Oscillator, speaker::Speaker},
};

pub mod number;
pub mod oscillator;
pub mod speaker;

// LYN: Snarl Node Impl

#[derive(Debug)]
pub enum PatchNode {
    // Signal
    Oscillator(Box<Oscillator>),
    Speaker(Speaker),
    // Communication

    // Logic

    // Variable
    Number(NumberNode),
    // Text(String),

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
}

#[derive(Debug, Clone, Copy)]
pub enum PatchNodeType {
    // Signal
    Oscillator,
    Speaker,

    // Variable
    Number,
    // Text,

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
}

pub trait PatchNodeTrait {
    fn name(&self) -> &str;
    fn inputs(&self) -> usize;
    fn outputs(&self) -> usize;
    fn pin_accept_multi(&self, pin_index: usize) -> bool;
    fn input_type(&self, pin_index: usize) -> PatchOutputType;
    fn output_type(&self, pin_index: usize) -> PatchOutputType;
    fn take_input(&mut self, pin_index: usize, source: OutPinId);
    fn drop_input(&mut self, pin_index: usize, source: OutPinId);
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
        }
    }
}

impl PatchNodeTrait for PatchNode {
    #[inline]
    fn name(&self) -> &str {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.name(),
            PatchNode::Speaker(speaker) => speaker.name(),

            // Variable
            PatchNode::Number(num) => num.name(),
            // PatchNode::Text(_) => "文字",

            // Calculation
            // PatchNode::Expression(_) => "表达式",
            // PatchNode::ADSRCurve(_) => "ADSR 曲线",
            // PatchNode::MidiToFreq => "MIDI 转频率",

            // Processing
            // PatchNode::WaveAdder(_) => "加波器",
            // PatchNode::WaveMultiplier(_) => "乘波器",
            // PatchNode::WaveOffseter(_) => "移幅器",
            // PatchNode::WaveScaler(_) => "倍幅器",
            // PatchNode::WaveClipper(_) => "限幅器",
        }
    }

    #[inline]
    fn inputs(&self) -> usize {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.inputs(),
            PatchNode::Speaker(speaker) => speaker.inputs(),

            // Variable
            PatchNode::Number(num) => num.inputs(),
        }
    }
    #[inline]
    fn outputs(&self) -> usize {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.outputs(),
            PatchNode::Speaker(speaker) => speaker.outputs(),

            // Variable
            PatchNode::Number(num) => num.outputs(),
        }
    }

    #[inline]
    fn pin_accept_multi(&self, pin_index: usize) -> bool {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.pin_accept_multi(pin_index),
            PatchNode::Speaker(speaker) => speaker.pin_accept_multi(pin_index),

            // Variable
            PatchNode::Number(num) => num.pin_accept_multi(pin_index),
        }
    }

    #[inline]
    fn input_type(&self, pin_index: usize) -> PatchOutputType {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.input_type(pin_index),
            PatchNode::Speaker(speaker) => speaker.input_type(pin_index),

            // Variable
            PatchNode::Number(num) => num.input_type(pin_index),
        }
    }

    #[inline]
    fn output_type(&self, pin_index: usize) -> PatchOutputType {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.output_type(pin_index),
            PatchNode::Speaker(speaker) => speaker.output_type(pin_index),

            // Variable
            PatchNode::Number(num) => num.output_type(pin_index),
        }
    }

    #[inline]
    fn take_input(&mut self, pin_index: usize, source: OutPinId) {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.take_input(pin_index, source),
            PatchNode::Speaker(speaker) => speaker.take_input(pin_index, source),

            // Variable
            PatchNode::Number(num) => num.take_input(pin_index, source),
        }
    }

    #[inline]
    fn drop_input(&mut self, pin_index: usize, source: OutPinId) {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.drop_input(pin_index, source),
            PatchNode::Speaker(speaker) => speaker.drop_input(pin_index, source),

            // Variable
            PatchNode::Number(num) => num.drop_input(pin_index, source),
        }
    }

    #[inline]
    fn output_block(&self, pin_index: usize) -> Option<&Block> {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.output_block(pin_index),
            PatchNode::Speaker(speaker) => speaker.output_block(pin_index),

            // Variable
            PatchNode::Number(num) => num.output_block(pin_index),
        }
    }

    #[inline]
    fn output_number(&self, pin_index: usize) -> Option<Number> {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.output_number(pin_index),
            PatchNode::Speaker(speaker) => speaker.output_number(pin_index),

            // Variable
            PatchNode::Number(num) => num.output_number(pin_index),
        }
    }

    #[inline]
    fn output_text(&self, pin_index: usize) -> Option<String> {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.output_text(pin_index),
            PatchNode::Speaker(speaker) => speaker.output_text(pin_index),

            // Variable
            PatchNode::Number(_) => None,
        }
    }
}

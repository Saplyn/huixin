use egui_snarl::OutPinId;

use crate::model::patch::{PatchOutput, PatchOutputType, node::speaker::Speaker};

use self::oscillator::Oscillator;

pub mod oscillator;
pub mod speaker;

// LYN: Snarl Node Impl

#[derive(Debug)]
pub enum PatchNode {
    // Signal
    Oscillator(Oscillator),
    Speaker(Speaker),
    // Communication

    // Logic

    // Variable
    // Number(Number),
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
    // Number,
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

impl PatchNode {
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            // Signal
            PatchNode::Oscillator(_) => "振荡器",
            PatchNode::Speaker(_) => "扬声器",
            // Variable
            // PatchNode::Number(_) => "数字",
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
    pub fn get_type(&self) -> PatchNodeType {
        match self {
            // Signal
            PatchNode::Oscillator(_) => PatchNodeType::Oscillator,
            PatchNode::Speaker(_) => PatchNodeType::Speaker,
            // Variable
            // PatchNode::Number(_) => PatchNodeType::Number,
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

    #[inline]
    pub fn inputs(&self) -> usize {
        match self {
            // Signal
            PatchNode::Oscillator(_) => Oscillator::INPUTS,
            PatchNode::Speaker(_) => Speaker::INPUTS,
        }
    }
    #[inline]
    pub fn outputs(&self) -> usize {
        match self {
            // Signal
            PatchNode::Oscillator(_) => Oscillator::OUTPUTS,
            PatchNode::Speaker(_) => Speaker::OUTPUTS,
        }
    }

    #[inline]
    pub fn pin_accept_multi(&self, pin_index: usize) -> bool {
        match self {
            // Signal
            PatchNode::Oscillator(_) => Oscillator::INPUT_ACCEPT_MULTI[pin_index],
            PatchNode::Speaker(_) => Speaker::INPUT_ACCEPT_MULTI[pin_index],
        }
    }

    #[inline]
    pub fn input_type(&self, pin_index: usize) -> PatchOutputType {
        match self {
            // Signal
            PatchNode::Oscillator(_) => Oscillator::INPUT_TYPE[pin_index],
            PatchNode::Speaker(_) => Speaker::INPUT_TYPE[pin_index],
        }
    }

    #[inline]
    pub fn output_type(&self, pin_index: usize) -> PatchOutputType {
        match self {
            // Signal
            PatchNode::Oscillator(_) => Oscillator::OUTPUT_TYPE[pin_index],
            PatchNode::Speaker(_) => Speaker::OUTPUT_TYPE[pin_index],
        }
    }

    pub fn add_input(&mut self, pin_index: usize, source: OutPinId) {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.set_input(pin_index, Some(source)),
            PatchNode::Speaker(speaker) => speaker.add_input(pin_index, source),
        }
    }

    pub fn del_input(&mut self, pin_index: usize, source: OutPinId) {
        match self {
            // Signal
            PatchNode::Oscillator(osc) => osc.set_input(pin_index, None),
            PatchNode::Speaker(speaker) => speaker.del_input(pin_index, source),
        }
    }

    pub fn output(&self, pin_index: usize) -> Option<PatchOutput> {
        // TODO:
        match self {
            // Signal
            PatchNode::Oscillator(osc) => {
                Some(PatchOutput::Block(Box::new(osc.output_block().to_owned())))
            }
            PatchNode::Speaker(_) => None,
        }
    }
}

use std::{f64::consts::PI, ops::RangeInclusive};

use egui_snarl::{NodeId, OutPinId};
use either::Either;
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

use crate::model::{
    data_mem::NonBlockData,
    patch::{Block, Number, WireDataType, node::PatchNodeTrait},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Oscillator {
    pub freq_or_seed: Number,
    pub phase: Number,
    pub waveform: Waveform,

    input_ids: [Option<OutPinId>; Self::INPUTS],

    #[serde(skip)]
    state: Option<Either<Number, StdRng>>,
    #[serde(skip, default = "WireDataType::empty_block")]
    memory: Block,
}

impl Oscillator {
    pub const NAME: &str = "振荡器";
    pub const INPUTS: usize = 4;
    pub const OUTPUTS: usize = 1;

    pub const INPUT_FREQ: usize = 0;
    pub const INPUT_PHASE: usize = 1;
    pub const INPUT_WAVEFORM: usize = 2;
    pub const INPUT_RESET: usize = 3;
    pub const INPUT_TYPE: [WireDataType; Self::INPUTS] = [
        WireDataType::Number,
        WireDataType::Number,
        WireDataType::Number,
        WireDataType::Bang,
    ];
    pub const INPUT_ACCEPT_MULTI: [bool; Self::INPUTS] = [false, false, false, false];

    pub const OUTPUT_BLOCK: usize = 0;
    pub const OUTPUT_TYPE: [WireDataType; Self::OUTPUTS] = [WireDataType::Block];

    pub const FREQ_RANGE: RangeInclusive<Number> = 0.0..=Number::MAX;
    pub const PHASE_RANGE: RangeInclusive<Number> = 0.0..=1.0;
    pub const WAVEFORM_RANGE: RangeInclusive<usize> = 0..=4;
}

impl PatchNodeTrait for Oscillator {
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
    fn output_block(&self, pin_index: usize) -> Option<&Block> {
        assert_eq!(pin_index, Self::OUTPUT_BLOCK);
        Some(&self.memory)
    }
    fn output_nonblock(&mut self, _pin_index: usize, _node_id: NodeId) -> Option<NonBlockData> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Waveform {
    Sine,
    Triangle,
    Saw,
    Square,
    Noise,
}

impl From<usize> for Waveform {
    fn from(value: usize) -> Self {
        match value {
            0 => Waveform::Sine,
            1 => Waveform::Triangle,
            2 => Waveform::Saw,
            3 => Waveform::Square,
            _ => Waveform::Noise,
        }
    }
}

impl Waveform {
    pub fn name(&self) -> &'static str {
        match self {
            Waveform::Sine => "正弦",
            Waveform::Triangle => "三角",
            Waveform::Saw => "锯齿",
            Waveform::Square => "方波",
            Waveform::Noise => "噪声",
        }
    }
}

impl Oscillator {
    pub fn new() -> Self {
        Self {
            freq_or_seed: 440.,
            phase: 0.,
            waveform: Waveform::Sine,

            input_ids: [None; Self::INPUTS],

            state: None,
            memory: WireDataType::empty_block(),
        }
    }
    pub fn reset(&mut self) {
        self.state = None;
    }
    pub fn next_block(&mut self, sample_rate: impl Into<Number>) -> Block {
        let step = self.freq_or_seed / sample_rate.into();
        let state = self
            .state
            .get_or_insert(if self.waveform == Waveform::Noise {
                Either::Right(StdRng::seed_from_u64(self.freq_or_seed as u64))
            } else {
                Either::Left(self.phase % 1.)
            });

        if self.waveform == Waveform::Noise
            && let Either::Left(_) = state
        {
            *state = Either::Right(StdRng::seed_from_u64(self.freq_or_seed as u64));
        }
        if self.waveform != Waveform::Noise
            && let Either::Right(_) = state
        {
            *state = Either::Left(self.phase % 1.);
        }

        let mut block = WireDataType::empty_block();
        for (index, frame) in block.iter_mut().enumerate() {
            match state {
                Either::Left(state) => {
                    *frame = match self.waveform {
                        Waveform::Sine => (2. * PI * *state).sin(),
                        Waveform::Triangle => 1. - 4. * (*state - 0.5).abs(),
                        Waveform::Saw => 2. * (*state) - 1.,
                        Waveform::Square => {
                            if *state < 0.5 {
                                1.
                            } else {
                                -1.
                            }
                        }
                        _ => unreachable!(),
                    };
                    *state = (*state + step) % 1.;
                }
                Either::Right(rng) => {
                    *frame = rng.random_range(-1.0..1.0);
                }
            };
            self.memory[index] = *frame;
        }

        block
    }
}

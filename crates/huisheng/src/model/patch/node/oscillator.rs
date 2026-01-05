use std::f64::consts::PI;

use egui_snarl::OutPinId;
use either::Either;
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

use crate::model::patch::{BLOCK_SIZE, Block, Number, PatchOutput, PatchOutputType};

#[derive(Debug, Serialize, Deserialize)]
pub struct Oscillator {
    pub freq_or_seed: Number,
    pub phase: Number,
    pub waveform: Waveform,

    input_ids: [Option<OutPinId>; Self::INPUTS],

    #[serde(skip)]
    state: Option<Either<Number, StdRng>>,
    #[serde(skip, default = "PatchOutput::empty_block")]
    memory: Block,
}

impl Oscillator {
    pub const INPUTS: usize = 4;
    pub const OUTPUTS: usize = 1;

    pub const INPUT_FREQ: usize = 0;
    pub const INPUT_PHASE: usize = 1;
    pub const INPUT_WAVEFORM: usize = 2;
    pub const INPUT_RESET: usize = 3;
    pub const INPUT_TYPE: [PatchOutputType; Self::INPUTS] = [
        PatchOutputType::Number,
        PatchOutputType::Number,
        PatchOutputType::Number,
        PatchOutputType::Bang,
    ];
    pub const INPUT_ACCEPT_MULTI: [bool; Self::INPUTS] = [false, false, false, false];

    pub const OUTPUT_BLOCK: usize = 0;
    pub const OUTPUT_TYPE: [PatchOutputType; Self::OUTPUTS] = [PatchOutputType::Block];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Waveform {
    Sine,
    Triangle,
    Saw,
    Square,
    Noise,
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
            memory: [0.; BLOCK_SIZE],
        }
    }
    pub fn reset(&mut self) {
        self.state = None;
    }
    pub fn set_input(&mut self, pin_index: usize, source: Option<OutPinId>) {
        self.input_ids[pin_index] = source;
    }
    pub fn output_block(&self) -> &Block {
        &self.memory
    }
    pub fn next_block(&mut self, sample_rate: Number) -> Block {
        let step = self.freq_or_seed / sample_rate;
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

        let mut block = [0.; BLOCK_SIZE];
        block.iter_mut().enumerate().for_each(|(index, frame)| {
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
        });

        block
    }
}

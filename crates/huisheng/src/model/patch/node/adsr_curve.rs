use egui_snarl::{NodeId, OutPinId};
use serde::{Deserialize, Serialize};

use crate::model::{
    data_mem::NonBlockData,
    patch::{BLOCK_SIZE, Block, Number, WireDataType, node::PatchNodeTrait},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ADSRCurve {
    pub attack: Number,
    pub decay: Number,
    pub sustain: Number,
    pub release: Number,

    pub peak: Number,
    pub keep: Number,

    input_ids: [Option<OutPinId>; Self::INPUTS],

    #[serde(skip)]
    retrigger: bool,
    #[serde(skip)]
    progress_in_millis: f64,
    #[serde(skip, default = "WireDataType::empty_block")]
    memory: Block,
}

impl ADSRCurve {
    pub const NAME: &str = "ADSR 曲线";
    pub const INPUTS: usize = 7;
    pub const OUTPUTS: usize = 1;

    pub const INPUT_ATTACK: usize = 0;
    pub const INPUT_DECAY: usize = 1;
    pub const INPUT_SUSTAIN: usize = 2;
    pub const INPUT_RELEASE: usize = 3;
    pub const INPUT_PEAK: usize = 4;
    pub const INPUT_KEEP: usize = 5;
    pub const INPUT_TRIGGER: usize = 6;
    pub const INPUT_TYPE: [WireDataType; Self::INPUTS] = [
        WireDataType::Number,
        WireDataType::Number,
        WireDataType::Number,
        WireDataType::Number,
        WireDataType::Number,
        WireDataType::Number,
        WireDataType::Bang,
    ];
    pub const INPUT_ACCEPT_MULTI: [bool; Self::INPUTS] =
        [false, false, false, false, false, false, false];

    pub const OUTPUT_BLOCK: usize = 0;
    pub const OUTPUT_TYPE: [WireDataType; Self::OUTPUTS] = [WireDataType::Block];
}

impl PatchNodeTrait for ADSRCurve {
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
    fn output_arbitrary(&mut self, _pin_index: usize, _node_id: NodeId) -> Option<NonBlockData> {
        None
    }
}

#[derive(Debug, PartialEq)]
enum Progress {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

impl ADSRCurve {
    fn progress(&self) -> Progress {
        let total_attack = self.attack;
        let total_decay = total_attack + self.decay;
        let total_sustain = total_decay + self.sustain;
        let total_release = total_sustain + self.release;
        if self.progress_in_millis < total_attack {
            Progress::Attack
        } else if self.progress_in_millis < total_decay {
            Progress::Decay
        } else if self.progress_in_millis < total_sustain {
            Progress::Sustain
        } else if self.progress_in_millis < total_release {
            Progress::Release
        } else {
            Progress::Finished
        }
    }
    fn should_stop_progressing(&self) -> bool {
        let stop_threadhold = self.attack + self.decay + self.sustain + self.release + 10000.;
        self.progress_in_millis > stop_threadhold
    }
}

impl ADSRCurve {
    pub fn new() -> Self {
        Self {
            attack: 0.,
            decay: 0.,
            sustain: 0.,
            release: 0.,

            peak: 1.,
            keep: 0.8,

            input_ids: [None; Self::INPUTS],

            retrigger: false,
            progress_in_millis: 0.,
            memory: [0.; BLOCK_SIZE],
        }
    }
    pub fn trigger(&mut self) {
        self.retrigger = true;
    }
    pub fn next_block(&mut self, sample_rate: u32) -> Block {
        let frame_duration_in_millis = 1000. / sample_rate as f64;

        if self.retrigger {
            self.retrigger = false;
            self.progress_in_millis = 0.;
        }

        let mut block = [0.; BLOCK_SIZE];
        for (index, frame) in block.iter_mut().enumerate() {
            let progress = self.progress();

            *frame = match progress {
                Progress::Attack => {
                    let value = (self.progress_in_millis / self.attack) * self.peak;
                    value.min(self.peak) as Number
                }
                Progress::Decay => {
                    let elapsed = self.progress_in_millis - self.attack;
                    let value = self.peak - (elapsed / self.decay) * (self.peak - self.keep);
                    value.max(self.keep) as Number
                }
                Progress::Sustain => self.keep,
                Progress::Release => {
                    let elapsed =
                        self.progress_in_millis - (self.attack + self.decay + self.sustain);
                    let value = self.keep - (elapsed / self.release) * self.keep;
                    value.max(0.) as Number
                }
                Progress::Finished => 0.,
            };

            if !self.should_stop_progressing() {
                self.progress_in_millis += frame_duration_in_millis;
            }

            self.memory[index] = *frame;
        }

        block
    }
}

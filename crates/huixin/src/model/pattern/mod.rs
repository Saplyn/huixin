use serde::{Deserialize, Serialize};

use crate::model::comm::SheetMessage;

use self::midi::MidiPattern;

pub mod curve;
pub mod event;
pub mod midi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetPatternType {
    Midi,
    // Curve,
    // Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SheetPattern {
    Midi(MidiPattern),
    // Curve(CurvePattern),
    // Event(EventPattern),
}

pub trait SheetPatternTrait {
    fn name_ref(&self) -> &String;
    fn icon_ref(&self) -> &String;
    fn color(&self) -> ecolor::Color32;
    fn usable(&self) -> bool;

    fn beats(&self) -> u64;
    fn msg_at(&self, tick: u64) -> Vec<SheetMessage>;
}

impl SheetPatternTrait for SheetPattern {
    #[inline]
    fn name_ref(&self) -> &String {
        match self {
            Self::Midi(pat) => pat.name_ref(),
        }
    }

    #[inline]
    fn icon_ref(&self) -> &String {
        match self {
            Self::Midi(pat) => pat.icon_ref(),
        }
    }

    #[inline]
    fn color(&self) -> ecolor::Color32 {
        match self {
            Self::Midi(pat) => pat.color(),
        }
    }

    #[inline]
    fn usable(&self) -> bool {
        match self {
            Self::Midi(pat) => pat.usable(),
        }
    }

    #[inline]
    fn beats(&self) -> u64 {
        match self {
            Self::Midi(pat) => pat.beats(),
        }
    }
    #[inline]
    fn msg_at(&self, tick: u64) -> Vec<SheetMessage> {
        match self {
            Self::Midi(pat) => pat.msg_at(tick),
        }
    }
}

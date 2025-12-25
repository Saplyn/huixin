use serde::{Deserialize, Serialize};

use crate::model::SheetMessage;

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
    fn name_mut(&mut self) -> &mut String;
    fn set_name(&mut self, name: String);

    fn icon_ref(&self) -> &String;
    fn icon_mut(&mut self) -> &mut String;
    fn set_icon(&mut self, icon: String);

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
    fn name_mut(&mut self) -> &mut String {
        match self {
            Self::Midi(pat) => pat.name_mut(),
        }
    }
    #[inline]
    fn set_name(&mut self, name: String) {
        match self {
            Self::Midi(pat) => pat.set_name(name),
        }
    }

    #[inline]
    fn icon_ref(&self) -> &String {
        match self {
            Self::Midi(pat) => pat.icon_ref(),
        }
    }
    #[inline]
    fn icon_mut(&mut self) -> &mut String {
        match self {
            Self::Midi(pat) => pat.icon_mut(),
        }
    }
    #[inline]
    fn set_icon(&mut self, icon: String) {
        match self {
            Self::Midi(pat) => pat.set_icon(icon),
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

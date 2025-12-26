use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::model::{comm::SheetMessage, state::CentralState, track::pattern::PatternTrack};

pub mod pattern;
pub mod timeline;

// LYN: Track

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetTrackType {
    Pattern,
    // Timeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SheetTrack {
    Pattern(PatternTrack),
    // Timeline(TimelineTrack),
}

pub trait SheetTrackTrait {
    fn name_ref(&self) -> &String;

    fn msg_at(&self, tick: u64, state: Arc<CentralState>) -> Vec<SheetMessage>;
}

impl SheetTrackTrait for SheetTrack {
    #[inline]
    fn name_ref(&self) -> &String {
        match self {
            Self::Pattern(track) => &track.name,
        }
    }
    #[inline]
    fn msg_at(&self, tick: u64, state: Arc<CentralState>) -> Vec<SheetMessage> {
        match self {
            Self::Pattern(track) => track.msg_at(tick, state),
        }
    }
}

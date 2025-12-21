use serde::{Deserialize, Serialize};

use crate::model::track::pattern::PatternTrack;

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

use crate::model::track::pattern::PatternTrack;

pub mod pattern;
pub mod timeline;

// LYN: Track

#[derive(Debug)]
pub enum SheetTrack {
    Pattern(PatternTrack),
    // Timeline(TimelineTrack),
}

use self::track::pattern::PatternTrack;

pub mod pattern;
pub mod track;

// LYN: Track

#[derive(Debug)]
pub enum SheetTrack {
    Pattern(PatternTrack),
    // Timeline(TimelineTrack),
}

// LYN: Message

#[derive(Debug)]
pub struct SheetMessage {
    pub tmp: String,
}

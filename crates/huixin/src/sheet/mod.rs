use std::sync::Weak;

use parking_lot::RwLock;

use crate::sheet::pattern::SheetPattern;

pub mod pattern;

// LYN: Track

#[derive(Debug)]
pub enum SheetTrack {
    Pattern(PatternTrack),
}

#[derive(Debug)]
pub struct PatternTrack {
    pub name: String,
    pub patterns: Vec<Weak<RwLock<SheetPattern>>>,
}

#[derive(Debug)]
pub struct EventTrack {
    pub name: String,
    // event vec, target
}

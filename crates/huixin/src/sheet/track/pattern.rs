use std::sync::Weak;

use parking_lot::RwLock;

use crate::sheet::pattern::SheetPattern;

#[derive(Debug)]
pub struct PatternTrack {
    pub name: String,
    pub patterns: Vec<Weak<RwLock<SheetPattern>>>,
}

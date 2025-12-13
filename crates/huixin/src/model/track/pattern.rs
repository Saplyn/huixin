use std::sync::Weak;

use parking_lot::RwLock;

use crate::model::pattern::SheetPattern;

#[derive(Debug)]
pub struct PatternTrack {
    pub name: String,
    pub patterns: Vec<Weak<RwLock<SheetPattern>>>,
}

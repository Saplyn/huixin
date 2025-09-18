use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;

use crate::{
    metronome::Metronome,
    sheet::{SheetPattern, SheetTrack},
};

#[derive(Debug)]
pub struct SheetReader {
    patterns: DashMap<String, SheetPattern>,
    tracks: RwLock<Vec<SheetTrack>>,
}

pub fn main(state: Arc<SheetReader>, metro: Arc<Metronome>) {
    loop {}
}

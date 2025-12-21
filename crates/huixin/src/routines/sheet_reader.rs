use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use dashmap::DashMap;
use log::info;
use lyn_util::egui::LynId;
use parking_lot::RwLock;

use crate::{
    app::{CommonState, PlayerContext},
    model::{
        SheetMessage,
        pattern::{SheetPattern, SheetPatternTrait, SheetPatternType, midi::MidiPattern},
        track::{SheetTrack, SheetTrackType, pattern::PatternTrack},
    },
    routines::{RoutineId, metronome::Metronome},
};

const REQUEST_TICK_POLL_INTERVAL: Duration = Duration::from_millis(50);

// LYN: Sheet Reader State Holder

#[derive(Debug)]
pub struct SheetReader {
    patterns: DashMap<String, Arc<RwLock<SheetPattern>>>,
    tracks: DashMap<String, Arc<RwLock<SheetTrack>>>,
}

impl SheetReader {
    #[inline]
    pub fn init() -> Self {
        Self {
            patterns: Default::default(),
            tracks: DashMap::new(),
        }
    }
    #[inline]
    pub fn main(
        state: Arc<Self>,
        common: Arc<CommonState>,
        metro: Arc<Metronome>,
        msg_tx: mpsc::Sender<SheetMessage>,
    ) -> ! {
        main(state, common, metro, msg_tx)
    }
}

// LYN: Sheet Reader Main Routine

fn main(
    state: Arc<SheetReader>,
    common: Arc<CommonState>,
    metro: Arc<Metronome>,
    msg_tx: mpsc::Sender<SheetMessage>,
) -> ! {
    info!("Sheet-reader started");

    loop {
        let Some(tick) = metro.request_tick(RoutineId::SheetReader) else {
            thread::sleep(REQUEST_TICK_POLL_INTERVAL);
            continue;
        };

        match common.player_context() {
            PlayerContext::Sheet => todo!(),
            PlayerContext::Pattern => {
                let Some(pat) = common.selected_pattern(state.clone()) else {
                    continue;
                };
                for msg in pat.read().msg_at(tick) {
                    msg_tx
                        .send(msg)
                        .expect("Instruction messaging channel unexpectedly closed");
                }
            }
        };
    }
}

// LYN: Sheet Reader Public APIs

impl SheetReader {
    pub fn add_track(&self, track_type: SheetTrackType) -> (String, Arc<RwLock<SheetTrack>>) {
        let track = Arc::new(RwLock::new(match track_type {
            SheetTrackType::Pattern => SheetTrack::Pattern(PatternTrack::new()),
        }));
        let id = LynId::obtain_string();
        self.tracks.insert(id.clone(), track.clone());
        (id, track)
    }

    pub fn del_track(&self, id: &String) -> Option<(String, Arc<RwLock<SheetTrack>>)> {
        self.tracks.remove(id)
    }

    pub fn get_track(&self, id: &String) -> Option<Arc<RwLock<SheetTrack>>> {
        self.tracks.get(id).map(|item| item.clone())
    }

    /// Returns an iterator over all tracks.
    pub fn tracks_iter(&self) -> dashmap::iter::Iter<'_, String, Arc<RwLock<SheetTrack>>> {
        self.tracks.iter()
    }

    /// Adds a new pattern with the given name and type.
    pub fn add_pattern(
        &self,
        pattern_type: SheetPatternType,
    ) -> (String, Arc<RwLock<SheetPattern>>) {
        let pat = Arc::new(RwLock::new(match pattern_type {
            SheetPatternType::Midi => SheetPattern::Midi(MidiPattern::new()),
        }));
        let id = LynId::obtain_string();
        self.patterns.insert(id.clone(), pat.clone());
        (id, pat)
    }

    /// Deletes the pattern with the given name.
    pub fn del_pattern(&self, id: &String) -> Option<(String, Arc<RwLock<SheetPattern>>)> {
        self.patterns.remove(id)
    }

    /// Retrieves a pattern by its ID.
    pub fn get_pattern(&self, id: &String) -> Option<Arc<RwLock<SheetPattern>>> {
        self.patterns.get(id).map(|item| item.clone())
    }

    /// Returns an iterator over all patterns.
    pub fn patterns_iter(&self) -> dashmap::iter::Iter<'_, String, Arc<RwLock<SheetPattern>>> {
        self.patterns.iter()
    }

    pub fn restore_state(
        &self,
        patterns: HashMap<String, SheetPattern>,
        tracks: HashMap<String, SheetTrack>,
    ) {
        self.patterns.clear();
        for (id, pattern) in patterns {
            self.patterns.insert(id, Arc::new(RwLock::new(pattern)));
        }
        self.tracks.clear();
        for (id, track) in tracks {
            self.tracks.insert(id, Arc::new(RwLock::new(track)));
        }
    }
}

use std::{
    sync::{Arc, Weak, mpsc},
    thread,
    time::Duration,
};

use dashmap::DashSet;
use log::{debug, error, info, warn};
use parking_lot::{RwLock, RwLockReadGuard};

use crate::{
    app::MainAppCmd,
    routines::{RoutineId, metronome::Metronome},
    sheet::{
        SheetTrack,
        pattern::{MidiPattern, SheetPattern, SheetPatternInner, SheetPatternType},
    },
};

const REQUEST_TICK_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct SheetReader {
    // core state
    context: RwLock<SheetContext>,
    patterns: RwLock<Vec<Arc<SheetPattern>>>,
    tracks: RwLock<Vec<SheetTrack>>,

    // api state
    pattern_names: DashSet<String>,
}

impl SheetReader {
    pub fn new() -> Self {
        Self {
            context: Default::default(),
            patterns: Default::default(),
            tracks: RwLock::new(Vec::new()),
            pattern_names: DashSet::default(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum SheetContext {
    #[default]
    Track,
    Pattern(Weak<SheetPattern>),
}

// LYN: Sheet Reader Main Routine

impl SheetReader {
    pub fn main(state: Arc<SheetReader>, metro: Arc<Metronome>, cmd_tx: mpsc::Sender<MainAppCmd>) {
        thread::spawn(|| main(state, metro)).join().unwrap_err();
        error!("Sheet-reader panicked");
        cmd_tx
            .send(MainAppCmd::ShowError(
                "Sheet-reader thread unexpectedly panicked".to_string(),
            ))
            .expect("Failed to request error to be displayed on UI");
    }
}

fn main(state: Arc<SheetReader>, metro: Arc<Metronome>) {
    info!("Sheet-reader started");

    loop {
        // wait for tick change
        if let Some(tick) = metro.request_tick(RoutineId::SheetReader) {
            debug!("{tick}");
        }
        thread::sleep(REQUEST_TICK_POLL_INTERVAL);
    }
}

// LYN: Sheet Reader Public APIs

impl SheetReader {
    pub fn context(&self) -> SheetContext {
        self.context.read().clone()
    }

    pub fn context_is_track(&self) -> bool {
        matches!(*self.context.read(), SheetContext::Track)
    }

    pub fn set_context(&self, context: SheetContext) {
        *self.context.write() = context;
    }

    pub fn patterns(&self) -> RwLockReadGuard<'_, Vec<Arc<SheetPattern>>> {
        self.patterns.read()
    }

    pub fn add_pattern(
        &self,
        name: String,
        pattern_type: SheetPatternType,
    ) -> Option<Arc<SheetPattern>> {
        if self.pattern_names.contains(&name) {
            return None;
        }

        let pattern = Arc::new(match pattern_type {
            SheetPatternType::Midi => SheetPattern {
                name: name.clone(),
                inner: SheetPatternInner::Midi(MidiPattern { notes: vec![] }),
            },
        });
        self.patterns.write().push(pattern.clone());
        self.pattern_names.insert(name);
        Some(pattern)
    }

    pub fn del_pattern(&self, name: String) {
        if self.pattern_names.remove(&name).is_some() {
            let mut patterns = self.patterns.write();
            if let Some(pos) = patterns.iter().position(|pat| pat.name == name) {
                patterns.remove(pos);
            }
        } else {
            warn!("Failed to delete pattern: {name} (not found)");
        }
    }

    pub fn add_track(&self) {}
}

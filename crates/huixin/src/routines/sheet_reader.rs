use std::{
    sync::{Arc, Weak, mpsc},
    thread,
    time::Duration,
};

use dashmap::DashSet;
use log::{debug, error, info, warn};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    app::{MainAppCmd, MainState},
    routines::{RoutineId, metronome::Metronome},
    sheet::{
        SheetTrack,
        pattern::{SheetPattern, SheetPatternTrait, SheetPatternType, midi::MidiPattern},
    },
};

const REQUEST_TICK_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct SheetReader {
    // core state
    context: RwLock<SheetContext>,
    patterns: RwLock<Vec<Arc<RwLock<SheetPattern>>>>,
    tracks: RwLock<Vec<RwLock<SheetTrack>>>,

    // api state
    pattern_names: DashSet<String>,

    // logic state
    main_state: Arc<MainState>,
}

impl SheetReader {
    pub fn new(main_state: Arc<MainState>) -> Self {
        Self {
            context: Default::default(),
            patterns: Default::default(),
            tracks: RwLock::new(Vec::new()),
            pattern_names: DashSet::default(),
            main_state,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum SheetContext {
    #[default]
    Track,
    Pattern(Weak<RwLock<SheetPattern>>),
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
        let Some(tick) = metro.request_tick(RoutineId::SheetReader) else {
            thread::sleep(REQUEST_TICK_POLL_INTERVAL);
            continue;
        };

        debug!("{tick}");
    }
}

// LYN: Sheet Reader Public APIs

impl SheetReader {
    /// Returns the current context of the sheet reader.
    pub fn context(&self) -> SheetContext {
        self.context.read().clone()
    }

    /// Returns true if the current context is set to `SheetContext::Track`.
    pub fn context_is_track(&self) -> bool {
        matches!(*self.context.read(), SheetContext::Track)
    }

    /// Sets the current context of the sheet reader.
    pub fn set_context(&self, context: SheetContext) {
        *self.context.write() = context;
    }

    /// Returns a readable guard to the list of patterns.
    pub fn patterns(&self) -> RwLockReadGuard<'_, Vec<Arc<RwLock<SheetPattern>>>> {
        self.patterns.read()
    }

    /// Returns a readable guard to the list of patterns.
    pub fn patterns_mut(&self) -> RwLockWriteGuard<'_, Vec<Arc<RwLock<SheetPattern>>>> {
        self.patterns.write()
    }

    /// Adds a new pattern with the given name and type.
    pub fn add_pattern(
        &self,
        name: String,
        pattern_type: SheetPatternType,
    ) -> Option<Arc<RwLock<SheetPattern>>> {
        if self.pattern_names.contains(&name) {
            return None;
        }

        let pattern = Arc::new(RwLock::new(match pattern_type {
            SheetPatternType::Midi => SheetPattern::Midi(MidiPattern::new(name.clone(), None)),
        }));
        self.patterns.write().push(pattern.clone());
        self.pattern_names.insert(name);
        Some(pattern)
    }

    /// Deletes the pattern with the given name.
    // TODO: maybe return a error type
    pub fn del_pattern(&self, name: String) -> Result<(), ()> {
        if self.pattern_names.remove(&name).is_some() {
            let mut patterns = self.patterns.write();
            if let Some(pos) = patterns
                .iter()
                .position(|pat| pat.read().name_ref() == &name)
            {
                patterns.remove(pos);
            }
            Ok(())
        } else {
            warn!("Failed to delete pattern: {name} (not found)");
            Err(())
        }
    }

    /// Adds a new track to the sheet.
    pub fn add_track(&self) {
        todo!() // TODO: impl this
    }
}

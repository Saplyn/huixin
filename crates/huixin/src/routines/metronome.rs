use std::{
    cmp,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use dashmap::DashMap;
use log::{debug, error, info};
use parking_lot::{RwLock, RwLockWriteGuard};

use crate::{app::MainAppCmd, routines::RoutineId};

pub const TICK_PER_BEAT: u32 = 4;
pub const SLEEP_PER_TICK: u32 = 50;
pub const MAX_SLEEP_TIME: Duration = Duration::from_millis(50);

// LYN: Metronome State Holder

#[derive(Debug)]
pub struct Metronome {
    // core states
    playing: RwLock<bool>,
    bpm: RwLock<f64>,
    curr_tick: RwLock<u64>,
    top_tick: RwLock<Option<u64>>,

    // api states
    tick_memory: DashMap<RoutineId, u64>,
}

impl Metronome {
    pub fn new() -> Self {
        Self {
            playing: RwLock::new(false),
            bpm: RwLock::new(130.),
            curr_tick: RwLock::new(0),
            top_tick: RwLock::new(None),
            tick_memory: DashMap::default(),
        }
    }
}

// LYN: Metronome Main Routine

impl Metronome {
    pub fn main(state: Arc<Metronome>, cmd_tx: mpsc::Sender<MainAppCmd>) {
        thread::spawn(|| main(state)).join().unwrap_err();
        error!("Metronome panicked");
        cmd_tx
            .send(MainAppCmd::ShowError(
                "Metronome thread unexpectedly panicked".to_string(),
            ))
            .expect("Failed to request error to be displayed on UI");
    }
}

fn main(state: Arc<Metronome>) -> ! {
    info!("Metronome started");
    let (mut interval, mut sleep_time) = bpm_to_tickable(*state.bpm.read());
    let mut remaining = interval;
    let mut active_bpm = *state.bpm.read();

    loop {
        // handle pause / play
        while !*state.playing.read() {
            thread::sleep(sleep_time);
        }

        // handle bpm change
        {
            let state_bpm_guard = state.bpm.read();
            if active_bpm != *state_bpm_guard {
                active_bpm = *state_bpm_guard;
                (interval, sleep_time) = bpm_to_tickable(*state_bpm_guard);
                remaining = cmp::min(remaining, interval);
            }
        }

        // sleep to next tick
        if remaining > sleep_time {
            thread::sleep(sleep_time);
            remaining -= sleep_time;
            continue;
        }
        thread::sleep(remaining);
        remaining = interval;

        // update tick
        {
            let top_tick_guard = state.top_tick.read();
            let mut curr_tick_guard = state.curr_tick.write();
            match *top_tick_guard {
                Some(top_tick) if *curr_tick_guard >= top_tick => *curr_tick_guard = 0,
                _ => *curr_tick_guard = curr_tick_guard.saturating_add(1),
            }
        }
        debug!("{}/{:?}", state.curr_tick.read(), state.top_tick.read());
    }
}

// LYN: Metronome Public APIs

impl Metronome {
    /// Returns whether the metronome is currently playing.
    pub fn playing(&self) -> bool {
        *self.playing.read()
    }

    /// Returns `true` if the metronome is fully stopped.
    pub fn stopped(&self) -> bool {
        *self.curr_tick.read() == 0 && !*self.playing.read()
    }

    /// Toggles the playing state of the metronome.
    pub fn toggle_playing(&self, value: Option<bool>) {
        let mut playing = self.playing.write();
        *playing = value.unwrap_or(!*playing);
    }

    /// Stops the metronome completely.
    pub fn stop(&self) {
        *self.playing.write() = false;
        *self.curr_tick.write() = 0;
        self.tick_memory.clear();
    }

    /// Returns the top tick, if any.
    pub fn top_tick(&self) -> Option<u64> {
        *self.top_tick.read()
    }

    /// Returns the BPM value.
    pub fn bpm(&self) -> f64 {
        *self.bpm.read()
    }

    /// Returns the current tick.
    pub fn query_tick(&self) -> u64 {
        *self.curr_tick.read()
    }

    /// Returns a writable guard to the BPM value.
    pub fn bpm_mut(&self) -> RwLockWriteGuard<'_, f64> {
        self.bpm.write()
    }

    /// Requests the current tick for the given routine.
    ///
    /// A routine may only receive a tick once, any subsequent requests within the same
    /// tick will return `None`. If the metronome is not playing, `None` is returned.
    /// To get the current tick without context and restrictions, use `.query_tick()`.
    pub fn request_tick(&self, id: RoutineId) -> Option<u64> {
        if !*self.playing.read() {
            return None;
        }

        let curr_tick = self.curr_tick.read();
        if let Some(last_tick) = self.tick_memory.get(&id)
            && *last_tick == *curr_tick
        {
            None
        } else {
            self.tick_memory.insert(id, *curr_tick);
            Some(*curr_tick)
        }
    }
}

// LYN: Helpers

fn bpm_to_tickable(bpm: f64) -> (Duration, Duration) {
    let interval = Duration::from_secs_f64(60. / (bpm * TICK_PER_BEAT as f64));
    (
        interval,
        cmp::min(MAX_SLEEP_TIME, interval / SLEEP_PER_TICK),
    )
}

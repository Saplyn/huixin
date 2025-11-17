use std::{
    cmp,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use log::{debug, error, info};
use parking_lot::RwLock;

use crate::app::MainAppCmd;

pub const TICK_PER_BEAT: u32 = 4;
pub const SLEEP_PER_TICK: u32 = 50;
pub const MAX_SLEEP_TIME: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct Metronome {
    pub playing: RwLock<bool>,
    pub bpm: RwLock<f64>,
    pub curr_tick: RwLock<u64>,
    pub top_tick: RwLock<Option<u64>>,
}

impl Metronome {
    pub fn new() -> Self {
        Self {
            playing: RwLock::new(false),
            bpm: RwLock::new(130.),
            curr_tick: RwLock::new(0),
            top_tick: RwLock::new(None),
        }
    }
}

// LYN: Main

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

// LYN: Helpers

fn bpm_to_tickable(bpm: f64) -> (Duration, Duration) {
    let interval = Duration::from_secs_f64(60. / (bpm * TICK_PER_BEAT as f64));
    (
        interval,
        cmp::min(MAX_SLEEP_TIME, interval / SLEEP_PER_TICK),
    )
}

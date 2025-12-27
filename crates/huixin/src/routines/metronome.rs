use std::{cmp, sync::Arc, thread, time::Duration};

use log::{info, trace};

use crate::model::state::CentralState;

pub const TICK_PER_BEAT: u64 = 4;
pub const SLEEP_PER_TICK: u32 = 50;
pub const MAX_SLEEP_TIME: Duration = Duration::from_millis(50);

// LYN: Metronome Main Routine

pub fn main(state: Arc<CentralState>) -> ! {
    info!("Metronome started");
    let bpm = state.sheet_bpm();
    let (mut interval, mut sleep_time) = bpm_to_tickable(bpm);
    let mut remaining = interval;
    let mut active_bpm = bpm;

    loop {
        // handle pause / play
        while !state.metro_playing() {
            thread::sleep(sleep_time);
        }

        // handle bpm change
        let state_bpm_guard = state.sheet_bpm();
        if active_bpm != state_bpm_guard {
            active_bpm = state_bpm_guard;
            (interval, sleep_time) = bpm_to_tickable(state_bpm_guard);
            remaining = cmp::min(remaining, interval);
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
            let limit = state.metro_tick_limit();
            let mut curr_tick_guard = state.metro_tick_mut();
            match limit {
                top_tick if *curr_tick_guard >= top_tick => *curr_tick_guard = 0,
                _ => *curr_tick_guard = curr_tick_guard.saturating_add(1),
            }
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

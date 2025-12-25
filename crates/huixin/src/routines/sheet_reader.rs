use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use log::info;

use crate::{
    app::PlayerContext,
    model::{
        SheetMessage, pattern::SheetPatternTrait, state::CentralState, track::SheetTrackTrait,
    },
    routines::RoutineId,
};

const REQUEST_TICK_POLL_INTERVAL: Duration = Duration::from_millis(50);

// LYN: Sheet Reader Main Routine

pub fn main(state: Arc<CentralState>, msg_tx: mpsc::Sender<SheetMessage>) -> ! {
    info!("Sheet-reader started");

    loop {
        let Some(tick) = state.metro_request_tick(RoutineId::SheetReader) else {
            thread::sleep(REQUEST_TICK_POLL_INTERVAL);
            continue;
        };

        match state.player_context() {
            PlayerContext::Sheet => {
                for track in state.sheet_tracks_iter() {
                    for msg in track.read().msg_at(tick, state.clone()) {
                        msg_tx
                            .send(msg)
                            .expect("Instruction messaging channel unexpectedly closed");
                    }
                }
            }
            PlayerContext::Pattern => {
                let Some(pat) = state.selected_pattern() else {
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

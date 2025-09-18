use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use dashmap::DashMap;
use log::{debug, error, info};
use parking_lot::RwLock;

use crate::{
    metronome::Metronome,
    sheet::{SheetPattern, SheetTrack},
    ui::main::UICmd,
};

const TICK_CHECK_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct SheetReader {
    patterns: DashMap<String, SheetPattern>,
    tracks: RwLock<Vec<SheetTrack>>,
}

impl SheetReader {
    pub fn new() -> Self {
        Self {
            patterns: DashMap::new(),
            tracks: RwLock::new(Vec::new()),
        }
    }
}

pub fn main(state: Arc<SheetReader>, metro: Arc<Metronome>, ui_cmd_tx: mpsc::Sender<UICmd>) {
    thread::spawn(|| actual_main(state, metro))
        .join()
        .unwrap_err();
    error!("Sheet-reader panicked");
    ui_cmd_tx
        .send(UICmd::ShowError(
            "Sheet-reader thread unexpectedly panicked".to_string(),
        ))
        .expect("Failed to request error to be displayed on UI");
}

fn actual_main(state: Arc<SheetReader>, metro: Arc<Metronome>) {
    info!("Sheet-reader started");
    let mut curr_tick = { *metro.curr_tick.read() };

    loop {
        // wait for tick change
        while curr_tick == *metro.curr_tick.read() {
            thread::sleep(TICK_CHECK_INTERVAL);
        }
        debug!("{curr_tick} -> {}", metro.curr_tick.read());
        curr_tick = *metro.curr_tick.read();
    }
}

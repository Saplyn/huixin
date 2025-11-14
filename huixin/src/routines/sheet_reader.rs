use std::{
    sync::{Arc, Weak, mpsc},
    thread,
    time::Duration,
};

use log::{debug, error, info};
use parking_lot::RwLock;

use crate::{
    apps::main::MainAppCmd,
    routines::metronome::Metronome,
    sheet::{SheetTrack, pattern::SheetPattern},
};

const TICK_CHECK_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct SheetReader {
    pub context: RwLock<SheetContext>,
    pub patterns: RwLock<Vec<Arc<SheetPattern>>>,
    pub tracks: RwLock<Vec<SheetTrack>>,
}

impl SheetReader {
    pub fn new() -> Self {
        Self {
            context: Default::default(),
            patterns: Default::default(),
            tracks: RwLock::new(Vec::new()),
        }
    }
}

#[derive(Debug, Default)]
pub enum SheetContext {
    #[default]
    Track,
    Pattern(Weak<SheetPattern>),
}

// LYN: Main

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

use std::{
    sync::{Arc, mpsc},
    thread,
};

use log::{error, info};

use crate::app::MainAppCmd;

// LYN: Instructor State Holder

#[derive(Debug)]
pub struct Instructor {}

impl Instructor {
    pub fn init() -> Self {
        Self {}
    }
}

// LYN: Instructor Main Routine

impl Instructor {
    pub fn main(state: Arc<Self>, cmd_tx: mpsc::Sender<MainAppCmd>) {
        thread::spawn(|| main(state)).join().unwrap_err();
        error!("Instructor panicked");
        cmd_tx
            .send(MainAppCmd::ShowError(
                "Instructor thread unexpectedly panicked".to_string(),
            ))
            .expect("Failed to request error to be displayed on UI");
    }
}

fn main(state: Arc<Instructor>) {
    info!("Instructor started");

    loop {}
}

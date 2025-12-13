use std::{sync::mpsc, thread, thread::JoinHandle, time::Duration};

use crate::{app::MainAppCmd, routines::RoutineId};

const CHECK_HEALTH_POLL_INTERVAL: Duration = Duration::from_millis(100);

// LYN: Guardian State Holder

#[derive(Debug)]
pub struct Guardian;

// LYN: Guardian Main Routine

impl Guardian {
    #[inline]
    pub fn main(routines: Vec<(RoutineId, JoinHandle<()>)>, cmd_tx: mpsc::Sender<MainAppCmd>) -> ! {
        loop {
            for (id, routine) in routines.iter() {
                if !routine.is_finished() {
                    continue;
                }
                cmd_tx
                    .send(MainAppCmd::ShowError(format!(
                        "Routine {id:?} unexpectedly panicked"
                    )))
                    .expect("Failed to request error to be displayed on UI");
            }
            thread::sleep(CHECK_HEALTH_POLL_INTERVAL);
        }
    }
}

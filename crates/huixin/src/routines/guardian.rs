use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{app::CommonState, routines::RoutineId};

const CHECK_HEALTH_POLL_INTERVAL: Duration = Duration::from_millis(100);

// LYN: Guardian State Holder

#[derive(Debug)]
pub struct Guardian;

// LYN: Guardian Main Routine

impl Guardian {
    #[inline]
    pub fn main(routines: Vec<(RoutineId, JoinHandle<()>)>, common: Arc<CommonState>) -> ! {
        loop {
            for (id, routine) in routines.iter() {
                if !routine.is_finished() {
                    continue;
                }
                *common.err_modal_message.write() = Some(format!("常驻线程 {id:?} 意外崩溃"));
            }
            thread::sleep(CHECK_HEALTH_POLL_INTERVAL);
        }
    }
}

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{model::state::CentralState, routines::RoutineId};

const CHECK_HEALTH_POLL_INTERVAL: Duration = Duration::from_millis(100);

// LYN: Guardian Main Routine

pub fn main(state: Arc<CentralState>, routines: Vec<(RoutineId, JoinHandle<()>)>) -> ! {
    loop {
        for (id, routine) in routines.iter() {
            if !routine.is_finished() {
                continue;
            }
            state.set_err_msg(Some(format!("常驻线程 {id:?} 意外崩溃")));
        }
        thread::sleep(CHECK_HEALTH_POLL_INTERVAL);
    }
}

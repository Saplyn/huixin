use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use log::{info, warn};

use crate::{model::SheetMessage, model::state::CentralState};

const NO_MSG_CHECK_HEALTH_INTERVAL: Duration = Duration::from_millis(50);

// LYN: Instructor Main Routine

pub fn main(state: Arc<CentralState>, msg_rx: mpsc::Receiver<SheetMessage>) -> ! {
    info!("Instructor started");

    loop {
        if let Ok(msg) = msg_rx.try_recv() {
            let Some(entry) = state.sheet_comm_targets_iter().get(&msg.target_id) else {
                continue;
            };
            let entry = entry.clone();
            state.worker_spawn_task(move || {
                // TODO: should I use `try_write()`
                let mut guard = entry.write();
                if guard.stream.is_none() {
                    guard.connect_stream();
                    return;
                }
                let format = guard.format;
                if let Err(err) = guard.stream.as_mut().unwrap().send(
                    dbg!(
                        msg.payload
                            .form_string(format)
                            .expect("Failed to serialize instruction payload")
                    )
                    .into(),
                ) {
                    warn!(
                        "Failed to send insturction payload to {}: {err}",
                        guard.addr
                    );
                    guard.connect_stream();
                }
            });
            continue;
        }

        // check target health
        for target_entry in state.sheet_comm_targets_iter() {
            if target_entry
                .try_read()
                .is_some_and(|guard| guard.stream.is_none())
            {
                let target_entry = target_entry.clone();
                state.worker_spawn_task(move || {
                    let mut guard = target_entry.write();
                    guard.connect_stream();
                });
            }
        }
        thread::sleep(NO_MSG_CHECK_HEALTH_INTERVAL);
    }
}

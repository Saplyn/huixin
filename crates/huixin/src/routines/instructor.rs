use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use log::{info, warn};

use crate::model::{
    comm::{CommTarget, SheetMessage},
    state::CentralState,
};

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
                let guard = entry.read();
                let (addr, format) = (guard.addr.clone(), guard.format);
                if guard.stream.is_none() {
                    drop(guard);
                    let stream = CommTarget::connect_stream_blocking(addr.as_str(), format);
                    entry.write().stream = stream;
                    return;
                }
                if let Err(err) = entry.write().stream.as_mut().unwrap().send(
                    msg.payload
                        .form_string(format)
                        .expect("Failed to serialize instruction payload")
                        .into(),
                ) {
                    warn!("Failed to send insturction payload to {}: {err}", addr);
                    let stream = CommTarget::connect_stream_blocking(addr.as_str(), format);
                    entry.write().stream = stream;
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
                    let (addr, format) = {
                        let guard = target_entry.read();
                        (guard.addr.clone(), guard.format)
                    };
                    let stream = CommTarget::connect_stream_blocking(addr.as_str(), format);
                    target_entry.write().stream = stream;
                });
            }
        }
        thread::sleep(NO_MSG_CHECK_HEALTH_INTERVAL);
    }
}

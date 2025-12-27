use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use log::{info, warn};

use crate::model::{comm::SheetMessage, state::CentralState};

const NO_MSG_CHECK_HEALTH_INTERVAL: Duration = Duration::from_millis(50);

// LYN: Instructor Main Routine

pub fn main(state: Arc<CentralState>, msg_rx: mpsc::Receiver<SheetMessage>) -> ! {
    info!("Instructor started");

    loop {
        if let Ok(msg) = msg_rx.try_recv() {
            let Some(entry) = state.sheet_get_comm_target(&msg.target_id) else {
                continue;
            };
            let entry = entry.clone();
            state.worker_spawn_task({
                let state = state.clone();
                move || {
                    let (id, addr, format) = {
                        let guard = entry.read();
                        (msg.target_id.clone(), guard.addr.clone(), guard.format)
                    };
                    if state.comm_get_stream(&id).is_none() {
                        state.comm_connect_stream_blocking(id, &addr, format);
                        return;
                    }
                    let data = msg
                        .payload
                        .form_string(format)
                        .expect("Failed to serialize instruction payload")
                        .into();
                    if let Err(err) = state.comm_send_data_blocking(&id, data) {
                        warn!("Failed to send insturction payload to {}: {err}", addr);
                        state.comm_connect_stream_blocking(id, &addr, format);
                    }
                }
            });
            continue;
        }

        // check target health
        for entry in state.sheet_comm_targets_iter() {
            let id = entry.key().clone();
            if state.comm_get_stream(&id).is_none() {
                let target_entry = entry.clone();
                state.worker_spawn_task({
                    let state = state.clone();
                    move || {
                        let (addr, format) = {
                            let guard = target_entry.read();
                            (guard.addr.clone(), guard.format)
                        };
                        state.comm_connect_stream_blocking(id, &addr, format);
                    }
                });
            }
        }
        thread::sleep(NO_MSG_CHECK_HEALTH_INTERVAL);
    }
}

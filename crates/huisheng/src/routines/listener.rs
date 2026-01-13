use std::{io, sync::Arc, thread, time::Duration};

use log::{info, trace, warn};

use crate::model::{data_mem::DataPack, state::CentralState};

const LISTENER_POLL_INTERVAL: Duration = Duration::from_millis(100);
const CONN_TIMEOUT_DURATION: Duration = Duration::from_secs(3);

pub fn main(state: Arc<CentralState>) -> ! {
    loop {
        if !state.port_listener_exists() {
            state.port_listener_start();
        }

        let listener_guard = state.port_listener_get();
        let Some(listener) = listener_guard.as_ref() else {
            thread::sleep(LISTENER_POLL_INTERVAL);
            continue;
        };

        if state.sheet_port().0 != listener.local_addr().unwrap().port() {
            state.port_listener_stop();
            thread::sleep(LISTENER_POLL_INTERVAL);
            continue;
        }

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let _ = stream.set_read_timeout(Some(CONN_TIMEOUT_DURATION));
                    let _ = stream.set_write_timeout(Some(CONN_TIMEOUT_DURATION));
                    trace!("{:?}", stream);
                    state.worker_spawn_task({
                        let state = state.clone();
                        move || {
                            if let Ok(ws) = ws::accept(stream) {
                                state.ws_streams_add(ws);
                            }
                        }
                    });
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No more incoming connections, break to poll next listener
                    break;
                }
                Err(e) => {
                    warn!("Error accepting connection: {}", e);
                }
            }
        }

        let mut to_be_removed = Vec::new();
        for mut entry in state.ws_streams_iter_mut() {
            let id = *entry.key();
            let stream = entry.value_mut();
            let msg = match stream.read() {
                Ok(msg) => msg,
                Err(ws::Error::Io(e)) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    warn!("WebSocket read error: {}", e);
                    to_be_removed.push(id);
                    continue;
                }
            };

            let str = match msg {
                ws::Message::Binary(bin_msg) => match String::from_utf8(bin_msg.to_vec()) {
                    Ok(str) => str,
                    Err(_) => continue,
                },
                ws::Message::Text(txt_msg) => txt_msg.to_string(),
                _ => {
                    continue;
                }
            };

            info!("{str}");
            if let Ok(pack) = json::from_str::<DataPack>(&str) {
                info!("{:?}", pack);
                for (key, data) in pack.data {
                    state.data_mem_set(pack.tag.clone(), key, data);
                }
            }
        }
        for id in to_be_removed {
            state.ws_streams_del(id);
        }

        thread::sleep(LISTENER_POLL_INTERVAL);
    }
}

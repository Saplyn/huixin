use std::{
    io::Write,
    net::TcpStream,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use dashmap::DashMap;
use log::{info, trace, warn};
use lyn_util::egui::LynId;
use parking_lot::RwLock;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::model::{CommTarget, SheetMessage};

const NO_MSG_CHECK_HEALTH_INTERVAL: Duration = Duration::from_millis(50);

// LYN: Instructor State Holder

#[derive(Debug)]
pub struct Instructor {
    targets: DashMap<LynId, Arc<RwLock<CommTarget>>>,

    workers: ThreadPool,
}

impl Instructor {
    #[inline]
    pub fn init() -> Self {
        Self {
            targets: DashMap::new(),
            workers: ThreadPoolBuilder::new().num_threads(4).build().unwrap(),
        }
    }
    #[inline]
    pub fn main(state: Arc<Self>, msg_rx: mpsc::Receiver<SheetMessage>) -> ! {
        main(state, msg_rx)
    }
}

// LYN: Instructor Main Routine

fn main(state: Arc<Instructor>, msg_rx: mpsc::Receiver<SheetMessage>) -> ! {
    info!("Instructor started");

    loop {
        if let Ok(msg) = msg_rx.try_recv() {
            let Some(entry) = state.targets().get(&msg.target_id) else {
                continue;
            };
            let entry = entry.clone();
            state.workers.spawn(move || {
                let mut guard = entry.upgradable_read();
                if guard.stream.is_none() {
                    guard.with_upgraded(|target| {
                        target.stream = TcpStream::connect(&target.addr).ok();
                    });
                    return;
                }
                if let Err(err) = guard.stream.as_ref().unwrap().write_all(
                    format!(
                        "{};",
                        ron::to_string(&msg.payload)
                            .expect("Failed to serialize instruction payload")
                    )
                    .as_bytes(),
                ) {
                    warn!(
                        "Failed to send insturction payload to {}: {err}",
                        guard.addr
                    );
                    guard.with_upgraded(|target| {
                        target.stream = TcpStream::connect(&target.addr).ok();
                    });
                }
            });
            continue;
        }

        // check target health
        for target_entry in state.targets() {
            if target_entry.read().stream.is_none() {
                let target_entry = target_entry.clone();
                state.workers.spawn(move || {
                    let mut guard = target_entry.write();
                    let stream = TcpStream::connect(&guard.addr);
                    trace!("{stream:?}");
                    guard.stream = stream.ok();
                });
            }
        }
        thread::sleep(NO_MSG_CHECK_HEALTH_INTERVAL);
    }
}

// LYN: Instructor Public APIs

impl Instructor {
    pub fn targets(&self) -> &DashMap<LynId, Arc<RwLock<CommTarget>>> {
        &self.targets
    }
}

use std::{
    collections::HashMap,
    fmt::Debug,
    io,
    net::{TcpListener, TcpStream},
    sync::{Arc, OnceLock},
};

use cpal::traits::{DeviceTrait, HostTrait};
use dashmap::{
    DashMap, DashSet,
    iter::{Iter, IterMut},
    mapref::one::{Ref, RefMut},
};
use lyn_util::{egui::LynId, types::WithId};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use rayon::{ThreadPool, ThreadPoolBuilder};
use serde::{Deserialize, Serialize};
use ws::WebSocket;

use crate::model::{data_mem::MemData, patch::Patch};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchId(String);
impl From<String> for PatchId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

const DEFAULT_LISTENER_PORT: u16 = 3000;

#[derive(Debug)]
pub struct CentralState {
    workers: ThreadPool,
    app: App,
    pub ui: UiState,
    pub cpal: CpalState,
    listener: ListenerState,
    sheet: Sheet,
}

#[derive(Debug)]
pub struct UiState {
    pub ctx: OnceLock<egui::Context>,
}

#[derive(Debug)]
pub struct App {
    err_modal_message: RwLock<Option<String>>,
    selected_patch: RwLock<Option<PatchId>>,
}

#[derive(Debug)]
pub struct ListenerState {
    port_listener: RwLock<Option<TcpListener>>,
    ws_streams: DashMap<LynId, WebSocket<TcpStream>>,
    data_memory: DashMap<String /* tag */, DashMap<String /* key */, MemData>>,
}

#[derive(Debug)]
pub struct Sheet {
    port: RwLock<(u16 /* port */, bool /* public */)>,

    patches: DashMap<PatchId, Arc<RwLock<Patch>>>,
    patches_ordering: RwLock<Vec<PatchId>>,
}

pub struct CpalState {
    pub host: cpal::Host,
    pub device: cpal::Device,
    pub supported_config: cpal::SupportedStreamConfig,
}

impl Debug for CpalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpalState")
            .field("host", &"{cpal::Host}")
            .field("device", &self.device.description())
            .field("supported_config", &self.supported_config)
            .finish()
    }
}

impl CentralState {
    pub fn init() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let supported_config = device
            .default_output_config()
            .expect("failed to get default output config");

        let ui = UiState {
            ctx: OnceLock::new(),
        };
        let cpal = CpalState {
            host,
            device,
            supported_config,
        };
        let sheet = Sheet {
            port: RwLock::new((DEFAULT_LISTENER_PORT, false)),

            patches: DashMap::new(),
            patches_ordering: RwLock::new(Vec::new()),
        };
        let listener = ListenerState {
            port_listener: RwLock::new(None),
            ws_streams: DashMap::new(),
            data_memory: DashMap::new(),
        };

        let app = App {
            err_modal_message: RwLock::new(None),
            selected_patch: RwLock::new(None),
        };

        Self {
            workers: ThreadPoolBuilder::new().build().unwrap(),
            ui,
            app,
            cpal,
            listener,
            sheet,
        }
    }
}

// LYN: State APIs

impl CentralState {
    pub fn worker_spawn_task(&self, f: impl FnOnce() + Send + 'static) {
        self.workers.spawn(f);
    }

    pub fn selected_patch_id(&self) -> RwLockReadGuard<'_, Option<PatchId>> {
        self.app.selected_patch.read()
    }
    pub fn selected_patch(&self) -> Option<WithId<PatchId, Arc<RwLock<Patch>>>> {
        let selected_id = self.app.selected_patch.read().clone()?;
        let patch = self.sheet.patches.get(&selected_id)?.clone();
        Some(WithId::new(selected_id, patch))
    }
    pub fn select_patch(&self, id: Option<PatchId>) {
        *self.app.selected_patch.write() = id;
    }

    pub fn app_set_err_msg(&self, msg: Option<String>) {
        *self.app.err_modal_message.write() = msg;
    }
    pub fn app_get_err_msg(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.app.err_modal_message.read()
    }
}

impl CentralState {
    pub fn port_listener_exists(&self) -> bool {
        self.listener.port_listener.read().is_some()
    }
    pub fn port_listener_get(&self) -> RwLockReadGuard<'_, Option<TcpListener>> {
        self.listener.port_listener.read()
    }
    pub fn port_listener_stop(&self) {
        self.listener.port_listener.write().take();
    }
    pub fn port_listener_start(&self) -> io::Result<()> {
        let (port, public) = *self.sheet.port.read();

        let addr = if public {
            format!("0.0.0.0:{}", port)
        } else {
            format!("127.0.0.1:{}", port)
        };
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        *self.listener.port_listener.write() = Some(listener);
        Ok(())
    }

    pub fn ws_streams_iter_mut(&self) -> IterMut<'_, LynId, WebSocket<TcpStream>> {
        self.listener.ws_streams.iter_mut()
    }
    pub fn ws_streams_get(&self, id: LynId) -> Option<Ref<'_, LynId, WebSocket<TcpStream>>> {
        self.listener.ws_streams.get(&id)
    }
    pub fn ws_streams_get_mut(&self, id: LynId) -> Option<RefMut<'_, LynId, WebSocket<TcpStream>>> {
        self.listener.ws_streams.get_mut(&id)
    }
    pub fn ws_streams_add(&self, ws: WebSocket<TcpStream>) {
        self.listener.ws_streams.insert(LynId::obtain(), ws);
    }
    pub fn ws_streams_del(&self, id: LynId) {
        self.listener.ws_streams.remove(&id);
    }

    pub fn data_mem_set(&self, tag: String, key: String, data: MemData) {
        let entry = self.listener.data_memory.entry(tag).or_default();
        match entry.entry(key) {
            dashmap::Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                if *entry == data {
                    entry.marker = entry.marker.wrapping_add(1);
                } else {
                    *entry = data;
                }
            }
            dashmap::Entry::Vacant(entry) => {
                entry.insert(data);
            }
        }
    }
    pub fn data_mem_get(&self, tag: &str, key: &str) -> Option<MemData> {
        self.listener
            .data_memory
            .get(tag)
            .and_then(|map| map.get(key).map(|entry| entry.clone()))
    }
}

impl CentralState {
    pub fn sheet_add_patch(&self) -> WithId<PatchId, Arc<RwLock<Patch>>> {
        let id: PatchId = LynId::obtain_string().into();
        let patch = Arc::new(RwLock::new(Patch::new()));
        self.sheet.patches.insert(id.clone(), patch.clone());
        self.sheet.patches_ordering.write().push(id.clone());
        WithId::new(id, patch)
    }
    pub fn sheet_get_patch(&self, id: &PatchId) -> Option<Arc<RwLock<Patch>>> {
        self.sheet.patches.get(id).map(|entry| entry.clone())
    }
    pub fn sheet_del_patch(&self, id: &PatchId) {
        self.sheet.patches_ordering.write().retain(|pid| pid != id);
        self.sheet.patches.remove(id);
    }
    pub fn sheet_patches_iter(&self) -> Iter<'_, PatchId, Arc<RwLock<Patch>>> {
        self.sheet.patches.iter()
    }
    pub fn sheet_patches_ordering_mut(&self) -> RwLockWriteGuard<'_, Vec<PatchId>> {
        self.sheet.patches_ordering.write()
    }

    pub fn sheet_port_mut(&self) -> RwLockWriteGuard<'_, (u16, bool)> {
        self.sheet.port.write()
    }
    pub fn sheet_port(&self) -> (u16, bool) {
        *self.sheet.port.read()
    }
}

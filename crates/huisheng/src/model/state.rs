use std::{
    collections::HashMap,
    fmt::Debug,
    io,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use cpal::traits::{DeviceTrait, HostTrait};
use dashmap::{
    DashMap,
    iter::{Iter, IterMut},
};
use lyn_util::{egui::LynId, project::Project};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use rayon::{ThreadPool, ThreadPoolBuilder};
use serde::{Deserialize, Serialize};
use ws::WebSocket;

use crate::model::{data_mem::MemData, patch::Patch};

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
    pub selecting_project_dir: RwLock<bool>,
}

#[derive(Debug)]
pub struct App {
    project: RwLock<Option<Project>>,
    states_loaded: RwLock<bool>,
    dsp_active: RwLock<bool>,
    err_modal_message: RwLock<Option<String>>,
    selected_patch_name: RwLock<Option<String>>,
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

    patches: DashMap<String /* name */, Arc<RwLock<Patch>>>,
    patches_ordering: RwLock<Vec<String /* name */>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedState {
    pub port: (u16 /* port */, bool /* public */),
    pub ordering: Vec<String /* name */>,
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
            selecting_project_dir: RwLock::new(false),
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
            project: RwLock::new(None),
            states_loaded: RwLock::new(false),
            dsp_active: RwLock::new(false),
            err_modal_message: RwLock::new(None),
            selected_patch_name: RwLock::new(None),
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
    pub fn load_project(&self, project_dir: PathBuf) -> Result<(), io::Error> {
        let project = Project::load(project_dir)?;
        self.app.project.write().replace(project);
        *self.app.states_loaded.write() = false;
        Ok(())
    }
    pub fn get_project(&self) -> RwLockReadGuard<'_, Option<Project>> {
        self.app.project.read()
    }
    pub fn close_project(&self) {
        self.app.project.write().take();
        *self.app.states_loaded.write() = false;
    }
    pub fn worker_spawn_task(&self, f: impl FnOnce() + Send + 'static) {
        self.workers.spawn(f);
    }

    pub fn selected_patch_name(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.app.selected_patch_name.read()
    }
    pub fn selected_patch(&self) -> Option<(String, Arc<RwLock<Patch>>)> {
        let selected_name = self.app.selected_patch_name.read().clone()?;
        let patch = self.sheet.patches.get(&selected_name)?.clone();
        Some((selected_name, patch))
    }
    pub fn select_patch(&self, name: Option<String>) {
        *self.app.selected_patch_name.write() = name;
    }

    pub fn dsp_active(&self) -> bool {
        *self.app.dsp_active.read()
    }
    pub fn dsp_value_mut(&self) -> RwLockWriteGuard<'_, bool> {
        self.app.dsp_active.write()
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
    pub fn sheet_patch_has_name(&self, name: &String) -> bool {
        self.sheet.patches.contains_key(name)
    }
    pub fn sheet_add_patch(&self, name: String) -> Arc<RwLock<Patch>> {
        let patch = Arc::new(RwLock::new(Patch::new()));
        self.sheet.patches.insert(name.clone(), patch.clone());
        self.sheet.patches_ordering.write().push(name);
        patch
    }
    pub fn sheet_get_patch(&self, name: &String) -> Option<Arc<RwLock<Patch>>> {
        self.sheet.patches.get(name).map(|entry| entry.clone())
    }
    pub fn sheet_del_patch(&self, name: &String) {
        self.sheet
            .patches_ordering
            .write()
            .retain(|pid| pid != name);
        self.sheet.patches.remove(name);
    }
    pub fn sheet_patches_iter(&self) -> Iter<'_, String, Arc<RwLock<Patch>>> {
        self.sheet.patches.iter()
    }
    pub fn sheet_patches_ordering_mut(&self) -> RwLockWriteGuard<'_, Vec<String>> {
        self.sheet.patches_ordering.write()
    }

    pub fn sheet_port_mut(&self) -> RwLockWriteGuard<'_, (u16, bool)> {
        self.sheet.port.write()
    }
    pub fn sheet_port(&self) -> (u16, bool) {
        *self.sheet.port.read()
    }

    pub fn states_loaded(&self) -> bool {
        *self.app.states_loaded.read()
    }
    pub fn sheet_to_persisted(&self) -> (HashMap<String, Arc<RwLock<Patch>>>, PersistedState) {
        let mut patches = HashMap::new();
        for entry in self.sheet.patches.iter() {
            patches.insert(entry.key().clone(), entry.value().clone());
        }

        (
            patches,
            PersistedState {
                port: *self.sheet.port.read(),
                ordering: self.sheet.patches_ordering.read().clone(),
            },
        )
    }
    pub fn sheet_from_persisted(
        &self,
        persisted: PersistedState,
        patches: HashMap<String, Arc<RwLock<Patch>>>,
    ) {
        *self.sheet.port.write() = persisted.port;

        self.sheet.patches.clear();
        for (name, patch) in patches {
            self.sheet.patches.insert(name, patch);
        }

        *self.sheet.patches_ordering.write() = persisted.ordering.clone();
        let mut patch_id_set: std::collections::HashSet<_> =
            self.sheet.patches.iter().map(|e| e.key().clone()).collect();
        for id in self.sheet.patches_ordering.read().iter() {
            patch_id_set.remove(id);
        }
        for id in patch_id_set {
            self.sheet.patches_ordering.write().push(id);
        }

        *self.app.states_loaded.write() = true;
    }
}

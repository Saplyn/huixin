use std::{fmt::Debug, sync::Arc};

use cpal::traits::{DeviceTrait, HostTrait};
use dashmap::DashMap;
use lyn_util::{egui::LynId, types::WithId};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};

use crate::{model::patch::Patch, routines::processor};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchId(String);
impl From<String> for PatchId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct CentralState {
    app: App,
    pub cpal: CpalState,
    sheet: Sheet,
}

#[derive(Debug)]
pub struct App {
    err_modal_message: RwLock<Option<String>>,
    selected_patch: RwLock<Option<PatchId>>,
}

#[derive(Debug)]
pub struct Sheet {
    patches_ordering: RwLock<Vec<PatchId>>,
    patches: DashMap<PatchId, Arc<RwLock<Patch>>>,
}

pub struct CpalState {
    pub host: cpal::Host,
    pub device: cpal::Device,
    pub supported_config: cpal::SupportedStreamConfig,
    pub config: cpal::StreamConfig,
}

impl Debug for CpalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpalState")
            .field("host", &"{cpal::Host}")
            .field("device", &self.device.description())
            .field("config", &self.config)
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
        let config = supported_config.clone().into();

        let cpal = CpalState {
            host,
            device,
            supported_config,
            config,
        };
        let sheet = Sheet {
            patches_ordering: RwLock::new(Vec::new()),
            patches: DashMap::new(),
        };

        let app = App {
            err_modal_message: RwLock::new(None),
            selected_patch: RwLock::new(None),
        };

        Self { app, cpal, sheet }
    }
}

// LYN: State APIs

impl CentralState {
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

    pub fn add_patch(&self) -> WithId<PatchId, Arc<RwLock<Patch>>> {
        let id: PatchId = LynId::obtain_string().into();
        let patch = Arc::new(RwLock::new(Patch::new()));
        self.sheet.patches.insert(id.clone(), patch.clone());
        self.sheet.patches_ordering.write().push(id.clone());
        WithId::new(id, patch)
    }
    pub fn get_patch(&self, id: &PatchId) -> Option<Arc<RwLock<Patch>>> {
        self.sheet.patches.get(id).map(|entry| entry.clone())
    }
    pub fn del_patch(&self, id: &PatchId) {
        self.sheet.patches_ordering.write().retain(|pid| pid != id);
        self.sheet.patches.remove(id);
    }
    pub fn patches_iter(&self) -> dashmap::iter::Iter<'_, PatchId, Arc<RwLock<Patch>>> {
        self.sheet.patches.iter()
    }
    pub fn patches_ordering_mut(&self) -> RwLockWriteGuard<'_, Vec<PatchId>> {
        self.sheet.patches_ordering.write()
    }

    pub fn app_set_err_msg(&self, msg: Option<String>) {
        *self.app.err_modal_message.write() = msg;
    }
    pub fn app_get_err_msg(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.app.err_modal_message.read()
    }
}

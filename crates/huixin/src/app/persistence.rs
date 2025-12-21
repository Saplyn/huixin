use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    APP_ID,
    app::CommonState,
    model::{CommTarget, pattern::SheetPattern, track::SheetTrack},
    routines::{instructor::Instructor, metronome::Metronome, sheet_reader::SheetReader},
};

// LYN: Persisted State

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedState {
    pub bpm: f64,

    pub patterns: HashMap<String, SheetPattern>,
    pub tracks: HashMap<String, SheetTrack>,
    pub targets: HashMap<String, CommTarget>,
}

pub fn form_persistable(
    common: Arc<CommonState>,
    metronome: Arc<Metronome>,
    sheet_reader: Arc<SheetReader>,
    instructor: Arc<Instructor>,
) -> PersistedState {
    let bpm = metronome.bpm();

    let mut patterns = HashMap::new();
    for entry in sheet_reader.patterns_iter() {
        patterns.insert(entry.key().clone(), entry.value().read().clone());
    }

    let mut tracks = HashMap::new();
    for entry in sheet_reader.tracks_iter() {
        tracks.insert(entry.key().clone(), entry.value().read().clone());
    }

    let mut targets = HashMap::new();
    for entry in instructor.targets() {
        targets.insert(entry.key().clone(), entry.value().read().clone());
    }

    PersistedState {
        bpm,
        patterns,
        tracks,
        targets,
    }
}

// LYN: Working Directory

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingDirectory(pub PathBuf);

impl WorkingDirectory {
    const STATE_DIR_NAME: &str = "states";
    pub fn state_path(&self, id: &str) -> PathBuf {
        self.0.join(Self::STATE_DIR_NAME).join(format!("{id}.json"))
    }
}

impl From<PathBuf> for WorkingDirectory {
    fn from(value: PathBuf) -> Self {
        Self(value)
    }
}

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

// LYN: EFrame Storage

pub struct AppStorage;

impl AppStorage {
    const STORAGE_PREFIX: &str = "lyn";

    pub fn key(key: impl AsRef<str>) -> String {
        format!("{}:{}", Self::STORAGE_PREFIX, key.as_ref())
    }
}

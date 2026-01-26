use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Project {
    pub project_dir: PathBuf,
    pub config: ProjectConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub info: ProjectInfo,
    pub dirs: Option<ProjectDirs>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub edition: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDirs {
    // sheet file name (huixin), without ext
    pub sheet: Option<String>,
    // patch directory name (huisheng)
    pub patch: Option<String>,
    // states directory name
    pub states: Option<String>,
}

impl Project {
    pub fn load(project_dir: PathBuf) -> io::Result<Self> {
        let config_str = fs::read_to_string(project_dir.join("project.toml"))?;
        let config = toml::from_str::<ProjectConfig>(&config_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Self {
            project_dir,
            config,
        })
    }
}

impl Project {
    const DEFAULT_SHEET_FILENAME: &str = "sheet";
    const DEFAULT_PATCH_DIRNAME: &str = "patches";
    const DEFAULT_STATES_DIRNAME: &str = "states";

    pub fn project_file(&self) -> PathBuf {
        self.project_dir.join("project.toml")
    }

    pub fn sheet_file(&self) -> PathBuf {
        let filename = self
            .config
            .dirs
            .as_ref()
            .and_then(|d| d.sheet.as_ref())
            .map(|s| s.as_str())
            .unwrap_or(Self::DEFAULT_SHEET_FILENAME);
        PathBuf::from(&self.project_dir).join(format!("{filename}.ron"))
    }

    pub fn patch_dir(&self) -> PathBuf {
        let dirname = self
            .config
            .dirs
            .as_ref()
            .and_then(|d| d.patch.as_ref())
            .map(|s| s.as_str())
            .unwrap_or(Self::DEFAULT_PATCH_DIRNAME);
        PathBuf::from(&self.project_dir).join(dirname)
    }
    pub fn patch_file(&self, name: &str) -> PathBuf {
        self.patch_dir().join(format!("{name}.ron"))
    }

    pub fn states_dir(&self) -> PathBuf {
        let dirname = self
            .config
            .dirs
            .as_ref()
            .and_then(|d| d.states.as_ref())
            .map(|s| s.as_str())
            .unwrap_or(Self::DEFAULT_STATES_DIRNAME);
        PathBuf::from(&self.project_dir).join(dirname)
    }
    pub fn state_file(&self, id: &str) -> PathBuf {
        self.states_dir().join(format!("{id}.ron"))
    }
}

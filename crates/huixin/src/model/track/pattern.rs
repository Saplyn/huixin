use serde::{Deserialize, Serialize};

use crate::model::pattern::SheetPattern;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternTrack {
    pub name: String,
    pub patterns: Vec<(u64, SheetPattern)>,
}

impl PatternTrack {
    pub fn new() -> Self {
        Self {
            name: String::from("未命名"),
            patterns: Vec::new(),
        }
    }
}

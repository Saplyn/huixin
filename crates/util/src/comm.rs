use std::ops;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub tag: String,
    pub data: ron::Map,
}

impl ops::Deref for Instruction {
    type Target = ron::Map;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl ops::DerefMut for Instruction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Instruction {
    #[inline]
    pub fn new(tag: String) -> Self {
        Self {
            tag,
            data: ron::Map::new(),
        }
    }
}

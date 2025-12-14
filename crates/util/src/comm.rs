use std::ops;

use serde::{Deserialize, Serialize};

pub type DataMap = json::Map<String, json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub tag: String,
    pub data: DataMap,
}

impl ops::Deref for Instruction {
    type Target = DataMap;
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
            data: DataMap::new(),
        }
    }

    pub fn form_string(&self) -> Result<String, json::Error> {
        json::to_string(&self)
    }
}

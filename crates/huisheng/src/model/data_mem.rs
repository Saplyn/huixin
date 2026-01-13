use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::patch::{Bang, BangMarker, Number, Text};

#[derive(Debug, Clone, Deserialize)]
pub struct DataPack {
    pub tag: String,
    pub data: HashMap<String, MemData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemData {
    pub marker: BangMarker,
    pub inner: NonBlockData,
}

impl<'de> Deserialize<'de> for MemData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(MemData {
            marker: 0,
            inner: NonBlockData::deserialize(deserializer)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NonBlockData {
    Number(Number),
    Text(Text),
    Bang(Bang),
}

impl MemData {
    pub fn get_number(&self) -> Option<f64> {
        if let NonBlockData::Number(n) = self.inner {
            Some(n)
        } else {
            None
        }
    }
    pub fn get_text(&self) -> Option<&str> {
        if let NonBlockData::Text(ref s) = self.inner {
            Some(s)
        } else {
            None
        }
    }
    pub fn get_marker(&self) -> u8 {
        self.marker
    }
}

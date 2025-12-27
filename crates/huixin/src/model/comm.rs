use std::net::TcpStream;

use lyn_util::comm::{Format, Instruction};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ws::WebSocket;

use crate::model::{DEFAULT_COMM_TARGET_ADDR, state::TargetId};

#[derive(Debug, Clone)]
pub struct SheetMessage {
    pub target_id: TargetId,
    pub payload: Instruction,
}

#[derive(Debug)]
pub enum CommStream {
    WebSocket(Box<WebSocket<TcpStream>>),
    TcpStream(TcpStream),
}

#[derive(Debug, Error)]
pub enum CommStreamErr {
    #[error("{0}")]
    WebSocket(#[from] ws::Error),
    #[error("{0}")]
    Std(#[from] std::io::Error),
    #[error("No communication stream connected")]
    NoCommStream,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommTarget {
    pub name: String,
    pub addr: String,
    pub format: Format,
}

impl Default for CommTarget {
    fn default() -> Self {
        Self {
            name: "未命名".to_string(),
            addr: DEFAULT_COMM_TARGET_ADDR.to_string(),
            format: Format::default(),
        }
    }
}

impl<'de> Deserialize<'de> for CommTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CommTargetDeser {
            name: String,
            addr: String,
            format: Format,
        }
        let deser = CommTargetDeser::deserialize(deserializer)?;
        Ok(CommTarget {
            name: deser.name,
            addr: deser.addr,
            format: deser.format,
        })
    }
}

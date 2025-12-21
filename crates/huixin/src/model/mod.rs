use std::net::TcpStream;

use lyn_util::comm::Instruction;
use serde::{Deserialize, Serialize};
use ws::{WebSocket, stream::MaybeTlsStream};

pub mod pattern;
pub mod track;

#[derive(Debug, Clone)]
pub struct SheetMessage {
    pub target_id: String,
    pub payload: Instruction,
}

const DEFAULT_COMM_TARGET_ADDR: &str = "ws://127.0.0.1:3000";

#[derive(Debug, Serialize)]
pub struct CommTarget {
    pub name: String,
    pub addr: String,
    #[serde(skip_serializing)]
    pub stream: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl Clone for CommTarget {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            addr: self.addr.clone(),
            stream: None,
        }
    }
}

impl Default for CommTarget {
    fn default() -> Self {
        Self {
            name: "未命名".to_string(),
            addr: DEFAULT_COMM_TARGET_ADDR.to_string(),
            stream: None,
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
        }
        let deser = CommTargetDeser::deserialize(deserializer)?;
        Ok(CommTarget {
            name: deser.name,
            addr: deser.addr,
            stream: None,
        })
    }
}

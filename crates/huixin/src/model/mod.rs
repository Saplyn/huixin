use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
};

use lyn_util::comm::{Format, Instruction};
use serde::{Deserialize, Serialize};
use ws::{WebSocket, stream::MaybeTlsStream};

use crate::model::state::TargetId;

pub mod pattern;
pub mod persistence;
pub mod state;
pub mod track;

#[derive(Debug, Clone)]
pub struct SheetMessage {
    pub target_id: TargetId,
    pub payload: Instruction,
}

const DEFAULT_COMM_TARGET_ADDR: &str = "127.0.0.1:3000";

#[derive(Debug)]
pub enum CommStream {
    WebSocket(Box<WebSocket<MaybeTlsStream<TcpStream>>>),
    TcpStream(TcpStream),
}

impl CommStream {
    pub fn send(&mut self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            CommStream::WebSocket(ws) => ws.send(data.into())?,
            CommStream::TcpStream(stream) => stream.write_all(&data)?,
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct CommTarget {
    pub name: String,
    pub addr: String,
    pub format: Format,
    #[serde(skip_serializing)]
    pub stream: Option<CommStream>,
}

impl CommTarget {
    pub fn connect_stream(&mut self) -> Option<()> {
        let addr: SocketAddr = self.addr.parse().ok()?;
        match self.format {
            Format::WsBasedJson => {
                let (ws, _) = ws::connect(format!("ws://{}", addr)).ok()?;
                self.stream = Some(CommStream::WebSocket(Box::new(ws)));
            }
            Format::TcpBasedOsc => {
                let stream = TcpStream::connect(addr).ok()?;
                self.stream = Some(CommStream::TcpStream(stream));
            }
        }
        Some(())
    }
}

impl Clone for CommTarget {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            addr: self.addr.clone(),
            format: self.format,
            stream: None,
        }
    }
}

impl Default for CommTarget {
    fn default() -> Self {
        Self {
            name: "未命名".to_string(),
            addr: DEFAULT_COMM_TARGET_ADDR.to_string(),
            format: Format::default(),
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
            format: Format,
        }
        let deser = CommTargetDeser::deserialize(deserializer)?;
        Ok(CommTarget {
            name: deser.name,
            addr: deser.addr,
            format: deser.format,
            stream: None,
        })
    }
}

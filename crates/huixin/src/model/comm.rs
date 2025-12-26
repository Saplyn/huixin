use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use lyn_util::comm::{Format, Instruction};
use serde::{Deserialize, Serialize};
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
    pub fn connect_stream_blocking(addr: &str, format: Format) -> Option<CommStream> {
        let addr: SocketAddr = addr.parse().ok()?;
        let timeout = Duration::from_secs(3);

        match format {
            Format::WsBasedJson => {
                let tcp_stream = TcpStream::connect_timeout(&addr, timeout).ok()?;
                tcp_stream.set_read_timeout(Some(timeout)).ok()?;
                tcp_stream.set_write_timeout(Some(timeout)).ok()?;

                let (ws, _) = ws::client(format!("ws://{}", addr), tcp_stream).ok()?;
                Some(CommStream::WebSocket(Box::new(ws)))
            }
            Format::TcpBasedOsc => {
                let stream = TcpStream::connect_timeout(&addr, timeout).ok()?;
                Some(CommStream::TcpStream(stream))
            }
        }
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

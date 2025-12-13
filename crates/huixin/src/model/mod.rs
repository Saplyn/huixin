use std::net::TcpStream;

use lyn_util::{comm::Instruction, egui::LynId};

pub mod pattern;
pub mod track;

#[derive(Debug, Clone)]
pub struct SheetMessage {
    pub target_id: LynId,
    pub payload: Instruction,
}

const DEFAULT_COMM_TARGET_ADDR: &str = "127.0.0.1:3000";

#[derive(Debug)]
pub struct CommTarget {
    pub name: String,
    pub addr: String,
    pub stream: Option<TcpStream>,
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

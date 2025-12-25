use std::{fmt::Display, ops};

use egui::IntoAtoms;
use osc::{OscMessage, OscPacket};
use serde::{Deserialize, Serialize};

pub type DataMap = json::Map<String, json::Value>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    #[default]
    WsBasedJson,
    TcpBasedOsc,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::WsBasedJson => write!(f, "绘心系列协议"),
            Format::TcpBasedOsc => write!(f, "PureData OSC"),
        }
    }
}

impl Format {
    pub fn variants() -> &'static [Format] {
        &[Format::WsBasedJson, Format::TcpBasedOsc]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub tag: String,
    pub data: DataMap,
    #[serde(skip)]
    pub format: Option<Format>,
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
    fn into_osc_packet(self) -> OscPacket {
        let args = self
            .data
            .into_iter()
            .filter_map(|(_, v)| match v {
                json::Value::Null => Some(osc::OscType::Nil),
                json::Value::Bool(b) => Some(osc::OscType::Bool(b)),
                json::Value::Number(n) => Some(if let Some(i) = n.as_i64() {
                    osc::OscType::Int(i as i32)
                } else if let Some(f) = n.as_f64() {
                    osc::OscType::Float(f as f32)
                } else {
                    return None;
                }),
                json::Value::String(s) => Some(osc::OscType::String(s)),
                _ => None,
            })
            .collect::<Vec<osc::OscType>>();

        OscPacket::Message(OscMessage {
            addr: self.tag,
            args,
        })
    }
}

impl Instruction {
    #[inline]
    pub fn form_string(self, format: Format) -> Option<String> {
        match format {
            Format::WsBasedJson => json::to_string(&self).ok(),
            Format::TcpBasedOsc => osc::encoder::encode(&self.into_osc_packet())
                .ok()
                .map(|pak| {
                    pak.iter()
                        .map(|b| b.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                        + ";"
                }),
        }
    }
}

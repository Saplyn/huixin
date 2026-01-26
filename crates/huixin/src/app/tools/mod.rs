use std::fmt::{Debug, Display};

pub mod connection_manager;
pub mod pattern_editor;
pub mod tester;

pub trait ToolWindow: Debug {
    fn tool_id(&self) -> ToolWindowId;
    fn icon(&self) -> String;
    fn window_open(&self) -> bool;
    fn window_open_mut(&mut self) -> &mut bool;
    fn toggle_open(&mut self, open: Option<bool>);
    fn draw(&mut self, ctx: &egui::Context);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolWindowId {
    PatternEditor,
    ConnectionManager,
    Tester,
}

impl ToolWindowId {
    pub fn to_store_key(&self) -> &'static str {
        match self {
            ToolWindowId::PatternEditor => "tool-pattern-editor",
            ToolWindowId::ConnectionManager => "tool-connection-manager",
            ToolWindowId::Tester => "tool-tester",
        }
    }
}

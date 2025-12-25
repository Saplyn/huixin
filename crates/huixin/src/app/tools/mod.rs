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

impl Display for ToolWindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ToolWindowId::PatternEditor => "Pattern Editor",
            ToolWindowId::ConnectionManager => "Connection Manager",
            ToolWindowId::Tester => "Tester",
        };
        write!(f, "{}", s)
    }
}

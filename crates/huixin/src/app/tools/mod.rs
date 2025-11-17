use std::fmt::Debug;

pub mod tester;

pub trait ToolWindow: Debug {
    fn icon(&self) -> String;
    fn window_open(&self) -> bool;
    fn window_open_mut(&mut self) -> &mut bool;
    fn toggle_open(&mut self, open: Option<bool>);
    fn draw(&mut self, ctx: &egui::Context);
}

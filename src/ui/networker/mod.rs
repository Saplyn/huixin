use crate::ui::helpers::{AppPage, PageId};

#[derive(Debug, Default)]
pub struct Networker {}

impl AppPage for Networker {
    fn page_id(&self) -> PageId {
        PageId::Networker
    }
}

impl eframe::App for Networker {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(self.page_id().to_string());
        });
    }
}

use crate::ui::helpers::{AppPage, PageId};

#[derive(Debug, Default)]
pub struct Composer;

impl AppPage for Composer {
    fn page_id(&self) -> PageId {
        PageId::Composer
    }
}

impl eframe::App for Composer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(self.page_id().to_string());
        });
    }
}

use crate::ui::helpers::{AppPage, PageId};

#[derive(Debug, Default)]
pub struct Programmer {}

impl AppPage for Programmer {
    fn page_id(&self) -> PageId {
        PageId::Programmer
    }
}

impl eframe::App for Programmer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(self.page_id().to_string());
        });
    }
}

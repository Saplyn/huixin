use crate::app::helpers::{AppPage, PageId};

#[derive(Debug, Default)]
pub struct Tester {}

impl AppPage for Tester {
    fn page_id(&self) -> PageId {
        PageId::Tester
    }
}

impl eframe::App for Tester {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {});

        egui::Window::new("Tester Window")
            .id(egui::Id::new("Tester Window"))
            .show(ctx, |ui| {
                ui.label("owo");

                let bg_stroke = ctx.style().visuals.widgets.noninteractive.bg_stroke;
            });
    }
}

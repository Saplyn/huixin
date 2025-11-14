use crate::apps::helpers::{AppPage, PageId};

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

        egui::Window::new("Tester Window 测试窗口")
            .id(egui::Id::new("Tester Window"))
            .show(ctx, |ui| {
                ui.label("test 测试");
                ui.code("test 测试");
            });
    }
}

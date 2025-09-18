use std::sync::Arc;

use crate::{
    sheet_reader::SheetReader,
    ui::helpers::{AppPage, PageId},
};

#[derive(Debug)]
pub struct Composer {
    pub sheet_reader: Arc<SheetReader>,
}

impl AppPage for Composer {
    fn page_id(&self) -> PageId {
        PageId::Composer
    }
}

impl eframe::App for Composer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("{:#?}", self.sheet_reader));
        });
    }
}

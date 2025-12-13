use std::sync::Arc;

use crate::{
    app::{CommonState, helpers::WidgetId, tools::ToolWindow},
    routines::{metronome::Metronome, sheet_reader::SheetReader},
};

#[derive(Debug)]
pub struct Tester {
    open: bool,
    common: Arc<CommonState>,
    metronome: Arc<Metronome>,
    sheet_reader: Arc<SheetReader>,
}

impl Tester {
    pub fn new(
        common: Arc<CommonState>,
        metronome: Arc<Metronome>,
        sheet_reader: Arc<SheetReader>,
    ) -> Self {
        Self {
            open: false,
            common,
            metronome,
            sheet_reader,
        }
    }
}

impl ToolWindow for Tester {
    fn icon(&self) -> String {
        "󰙨 ".to_string()
    }

    fn window_open(&self) -> bool {
        self.open
    }

    fn window_open_mut(&mut self) -> &mut bool {
        &mut self.open
    }

    fn toggle_open(&mut self, open: Option<bool>) {
        if let Some(open) = open {
            self.open = open;
        } else {
            self.open = !self.open;
        }
    }

    fn draw(&mut self, ctx: &egui::Context) {
        egui::Window::new("Tester")
            .id(WidgetId::Tester.into())
            .frame(egui::Frame::window(&ctx.style()).inner_margin(0))
            .title_bar(false)
            .min_size(emath::vec2(400., 150.))
            .show(ctx, |ui| {
                egui::TopBottomPanel::top(WidgetId::TesterTopUtilBar).show_inside(ui, |ui| {
                    self.top_util_bar(ui);
                });
                egui::SidePanel::right(WidgetId::TesterRightDetailPanel)
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        self.right_detail_panel(ui);
                    });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    self.central_panel(ui);
                });
            });
    }
}

impl Tester {
    fn top_util_bar(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(" ")
                        .frame(false)
                        .frame_when_inactive(false),
                )
                .clicked()
            {
                self.open = false;
            }
            ui.disable();
            ui.vertical_centered(|ui| {
                ui.heading("Tester");
            });
        });
    }

    fn right_detail_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("right");
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.code(format!(
                "*selected_pattern: {:#?}",
                self.common
                    .selected_pattern
                    .read()
                    .as_ref()
                    .and_then(|weak| weak.upgrade())
            ));
            ui.code(format!("{:#?}", self.common));
            ui.separator();
            ui.code(format!("{:#?}", self.sheet_reader));
            ui.separator();
            ui.code(format!("{:#?}", self.metronome));
        });
    }
}

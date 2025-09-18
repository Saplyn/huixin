use std::{ops::DerefMut, sync::mpsc};

use crate::{
    app::App,
    ui::{
        helpers::{AppPage, PageId, WidgetId},
        main::{error_modal::ErrorModal, performance::Performance},
    },
};

mod error_modal;
mod performance;

// LYN: UI State Holder

#[derive(Debug)]
pub struct UI {
    pub cmd_tx: mpsc::Sender<UICmd>,
    pub cmd_rx: mpsc::Receiver<UICmd>,
    pub active_page: PageId,
    pub pages: Vec<Box<dyn AppPage>>,
    pub performance: Performance,
    pub error_modal: ErrorModal,
}

// LYN: UI Command

pub enum UICmd {
    ShowError(String),
}

impl App {
    fn handle_ui_cmd(&mut self) {
        let cmd = match self.ui.cmd_rx.try_recv() {
            Ok(cmd) => cmd,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.ui
                    .error_modal
                    .set_msg("UI command channel unexpectedly closed".to_string());
                return;
            }
        };

        match cmd {
            UICmd::ShowError(msg) => self.ui.error_modal.set_msg(msg),
        }
    }
}

// LYN: UI Implementation

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.ui
            .performance
            .update_frame_history(ctx.input(|i| i.time), frame.info().cpu_usage);

        self.handle_ui_cmd();

        self.draw_ui(ctx, frame);
        self.draw_active_app(ctx, frame);
        self.ui.error_modal.try_draw(ctx);
    }
}

impl App {
    fn draw_ui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top(WidgetId::MainAppTopToolBar).show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.draw_app_menu(ui);
                ui.separator();
                self.draw_media_control(ui);
                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.draw_navigation(ui);
                    ui.separator();
                });
            })
        });

        egui::TopBottomPanel::bottom(WidgetId::MainAppButtonStatusBar).show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.ui.performance.draw(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(env!("BUILD_INFO"));
                });
            });
        });
    }

    // TODO:
    fn draw_app_menu(&mut self, ui: &mut egui::Ui) {
        ui.label("menubar");
    }

    // TODO:
    fn draw_media_control(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(self.metronome.playing.write().deref_mut(), "Playing");
        ui.add(egui::DragValue::new(self.metronome.bpm.write().deref_mut()).range(1..=640));
    }

    fn draw_navigation(&mut self, ui: &mut egui::Ui) {
        let active_page = self.ui.active_page;
        self.ui.pages.iter_mut().rev().for_each(|page| {
            if ui
                .selectable_label(page.page_id() == active_page, page.page_id().to_string())
                .clicked()
            {
                self.ui.active_page = page.page_id();
            }
        });
    }

    fn draw_active_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        for page in self.ui.pages.iter_mut() {
            if page.page_id() == self.ui.active_page
                || ctx.memory(|mem| mem.everything_is_visible())
            {
                page.update(ctx, frame);
            }
        }
    }
}

use std::{
    ops::DerefMut,
    sync::{Arc, mpsc},
    thread,
};

use crate::{
    app_ui::{
        helpers::WidgetId,
        widgets::{error_modal::ErrorModal, performance::Performance},
    },
    routines::{
        metronome::Metronome,
        sheet_reader::{SheetContext, SheetReader},
    },
};

mod helpers;
mod widgets;

// LYN: Main App State Holder

#[derive(Debug)]
pub struct MainApp {
    // command handling
    pub cmd_tx: mpsc::Sender<MainAppCmd>,
    pub cmd_rx: mpsc::Receiver<MainAppCmd>,

    // widget states
    pub performance: Performance,
    pub error_modal: ErrorModal,

    // routine states
    pub metronome: Arc<Metronome>,
    pub sheet_reader: Arc<SheetReader>,
}

impl MainApp {
    pub fn prepare() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let metronome = Arc::new(Metronome::new());
        let sheet_reader = Arc::new(SheetReader::new());

        thread::spawn({
            let state = metronome.clone();
            let cmd_tx = cmd_tx.clone();
            move || Metronome::main(state, cmd_tx)
        });
        thread::spawn({
            let state = sheet_reader.clone();
            let metro = metronome.clone();
            let cmd_tx = cmd_tx.clone();
            move || SheetReader::main(state, metro, cmd_tx)
        });

        Self {
            cmd_tx,
            cmd_rx,
            performance: Default::default(),
            error_modal: Default::default(),
            metronome,
            sheet_reader,
        }
    }
}

// LYN: Main App Command

pub enum MainAppCmd {
    ShowError(String),
}

impl MainApp {
    fn handle_ui_cmd(&mut self) {
        let cmd = match self.cmd_rx.try_recv() {
            Ok(cmd) => cmd,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.error_modal
                    .set_msg("UI command channel unexpectedly closed".to_string());
                return;
            }
        };

        match cmd {
            MainAppCmd::ShowError(msg) => self.error_modal.set_msg(msg),
        }
    }
}

// LYN: UI Implementation

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.performance
            .update_frame_history(ctx.input(|i| i.time), frame.info().cpu_usage);

        self.handle_ui_cmd();

        self.draw_ui(ctx);
        self.draw_active_tool_windows(ctx);
        self.error_modal.try_draw(ctx);
    }
}

impl MainApp {
    fn draw_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top(WidgetId::MainAppTopToolBar).show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.app_menu(ui);
                ui.separator();
                self.context_control(ui);
                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.toolbar(ui);
                    ui.separator();
                });
            })
        });

        egui::TopBottomPanel::bottom(WidgetId::MainAppButtonStatusBar).show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.performance.ui(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(env!("BUILD_INFO"));
                });
            });
        });

        egui::SidePanel::left(WidgetId::MainAppLeftExplorerPanel)
            .show(ctx, |ui| ui.label("explorer"));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Tracks");
        });
    }

    fn draw_active_tool_windows(&mut self, ctx: &egui::Context) {
        egui::Window::new("Saplyn 苗傲")
            .id(egui::Id::new("Saplyn"))
            .title_bar(false)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("top").show_inside(ui, |ui| ui.label("top"));
                ui.label("Meow 喵喵");
                ui.allocate_space(ui.available_size());
            });
    }

    // TODO:
    fn app_menu(&mut self, ui: &mut egui::Ui) {
        ui.label("menubar");
    }

    // TODO:
    fn context_control(&mut self, ui: &mut egui::Ui) {
        ui.label(match *self.sheet_reader.context.read() {
            SheetContext::Track => "Track".to_string(),
            SheetContext::Pattern(ref pat) => pat
                .upgrade()
                .map(|pat| pat.name.clone())
                .unwrap_or("ERROR".to_string()),
        });
        ui.checkbox(self.metronome.playing.write().deref_mut(), "Playing");
        ui.add(egui::DragValue::new(self.metronome.bpm.write().deref_mut()).range(1..=640));
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.label("Toolbar");
    }
}

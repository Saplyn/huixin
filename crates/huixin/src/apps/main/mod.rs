use std::{
    ops::DerefMut,
    sync::{Arc, mpsc},
    thread,
};

use parking_lot::RwLock;

use crate::{
    apps::{
        composer::Composer,
        helpers::{AppPage, PageId, WidgetId},
        main::{error_modal::ErrorModal, performance::Performance},
        networker::Networker,
        programmer::Programmer,
        tester::Tester,
    },
    routines::{
        metronome::{self, Metronome},
        sheet_reader::{SheetContext, SheetReader},
    },
    sheet::{
        Timed,
        pattern::{PianoNote, PianoPattern, SheetPattern, SheetPatternInner},
    },
};

mod error_modal;
mod performance;

// LYN: Main App State Holder

#[derive(Debug)]
pub struct MainApp {
    pub cmd_tx: mpsc::Sender<MainAppCmd>,
    pub cmd_rx: mpsc::Receiver<MainAppCmd>,
    pub active_page: PageId,
    pub pages: Vec<Box<dyn AppPage>>,
    pub performance: Performance,
    pub error_modal: ErrorModal,

    pub metronome: Arc<Metronome>,
    pub sheet_reader: Arc<SheetReader>,
}

impl MainApp {
    pub fn prepare() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let metronome = Arc::new(Metronome::new());
        let sheet_reader = Arc::new(SheetReader::new());

        // TODO: placeholder
        {
            let pattern = Arc::new(SheetPattern {
                name: "Test".to_string(),
                inner: SheetPatternInner::Piano(PianoPattern {
                    notes: vec![
                        Timed::new(
                            0,
                            PianoNote {
                                strength: u16::MAX,
                                code: 60,
                                length: metronome::TICK_PER_BEAT as u64,
                            },
                        ),
                        Timed::new(
                            (metronome::TICK_PER_BEAT * 2) as u64,
                            PianoNote {
                                strength: u16::MAX,
                                code: 61,
                                length: metronome::TICK_PER_BEAT as u64,
                            },
                        ),
                    ],
                }),
            });
            sheet_reader.patterns.write().push(pattern.clone());
            *sheet_reader.context.write() = SheetContext::Pattern(Arc::downgrade(&pattern));
        }

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
            pages: vec![
                Box::new(Tester {}),
                Box::new(Composer {
                    sheet_reader: sheet_reader.clone(),
                }),
                Box::new(Programmer {}),
                Box::new(Networker {}),
            ],
            cmd_tx,
            cmd_rx,
            active_page: Default::default(),
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
        self.draw_active_app(ctx, frame);
        self.error_modal.try_draw(ctx);
    }
}

impl MainApp {
    fn draw_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top(WidgetId::MainAppTopToolBar).show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.app_menu(ui);
                ui.separator();
                self.media_control(ui);
                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.navigation(ui);
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
    }

    // TODO:
    fn app_menu(&mut self, ui: &mut egui::Ui) {
        ui.label("menubar");
    }

    // TODO:
    fn media_control(&mut self, ui: &mut egui::Ui) {
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

    fn navigation(&mut self, ui: &mut egui::Ui) {
        let active_page = self.active_page;
        self.pages.iter_mut().rev().for_each(|page| {
            if ui
                .selectable_label(page.page_id() == active_page, page.page_id().to_string())
                .clicked()
            {
                self.active_page = page.page_id();
            }
        });
    }

    fn draw_active_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        for page in self.pages.iter_mut() {
            if page.page_id() == self.active_page || ctx.memory(|mem| mem.everything_is_visible()) {
                page.update(ctx, frame);
            }
        }
    }
}

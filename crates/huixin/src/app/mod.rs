use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Weak, mpsc},
    thread,
};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    app::{
        helpers::WidgetId,
        tools::{ToolWindow, pattern_editor::PatternEditor, tester::Tester},
        widgets::{error_modal::ErrorModal, performance::Performance},
    },
    routines::{
        metronome::{Metronome, TICK_PER_BEAT},
        sheet_reader::{SheetContext, SheetReader},
    },
    sheet::{
        Timed,
        pattern::{MidiNote, MidiPattern, SheetPattern, SheetPatternInner, SheetPatternType},
    },
};

mod helpers;
mod tools;
mod widgets;

// LYN: Main App State Holder

#[derive(Debug)]
pub struct MainState {
    selected_pattern: RwLock<Option<Weak<RwLock<SheetPattern>>>>,
}

impl MainState {
    pub fn selected_pattern(&self) -> RwLockReadGuard<'_, Option<Weak<RwLock<SheetPattern>>>> {
        self.selected_pattern.read()
    }
    pub fn selected_pattern_mut(&self) -> RwLockWriteGuard<'_, Option<Weak<RwLock<SheetPattern>>>> {
        self.selected_pattern.write()
    }
    pub fn select_pattern(&self, pattern: Option<Weak<RwLock<SheetPattern>>>) {
        *self.selected_pattern.write() = pattern;
    }
}

#[derive(Debug)]
pub struct MainApp {
    // command handling
    pub cmd_tx: mpsc::Sender<MainAppCmd>,
    pub cmd_rx: mpsc::Receiver<MainAppCmd>,

    // widget states
    pub performance: Performance,
    pub error_modal: ErrorModal,
    pub tools: Vec<Box<dyn ToolWindow>>,

    // routine states
    pub main_state: Arc<MainState>,
    pub metronome: Arc<Metronome>,
    pub sheet_reader: Arc<SheetReader>,
}

impl MainApp {
    pub fn prepare() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let metronome = Arc::new(Metronome::new());
        let sheet_reader = Arc::new(SheetReader::new());

        let main_state = Arc::new(MainState {
            selected_pattern: RwLock::new(None),
        });

        let tools: Vec<Box<dyn ToolWindow>> = vec![
            Box::new(Tester::new(
                main_state.clone(),
                metronome.clone(),
                sheet_reader.clone(),
            )),
            Box::new(PatternEditor::new(
                main_state.clone(),
                metronome.clone(),
                sheet_reader.clone(),
            )),
        ];

        // FIXME: test data
        sheet_reader.add_pattern("Test 1".to_string(), SheetPatternType::Midi);
        sheet_reader.add_pattern("Test 2".to_string(), SheetPatternType::Midi);
        sheet_reader.add_pattern("Test 3".to_string(), SheetPatternType::Midi);
        sheet_reader.patterns_mut().deref_mut()[0].write().inner =
            SheetPatternInner::Midi(MidiPattern {
                notes: vec![
                    Timed::new(
                        0,
                        MidiNote {
                            strength: u16::MAX,
                            midicode: 60,
                            length: TICK_PER_BEAT,
                        },
                    ),
                    Timed::new(
                        TICK_PER_BEAT,
                        MidiNote {
                            strength: u16::MAX,
                            midicode: 61,
                            length: TICK_PER_BEAT,
                        },
                    ),
                    Timed::new(
                        2 * TICK_PER_BEAT,
                        MidiNote {
                            strength: u16::MAX,
                            midicode: 62,
                            length: TICK_PER_BEAT,
                        },
                    ),
                    Timed::new(
                        3 * TICK_PER_BEAT,
                        MidiNote {
                            strength: u16::MAX,
                            midicode: 63,
                            length: TICK_PER_BEAT,
                        },
                    ),
                ],
            });

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
            tools,
            main_state,
            metronome,
            sheet_reader,
        }
    }
}

// LYN: Main App Command Handling

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

// LYN: Main App UI Implementation

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.performance
            .update_frame_history(ctx.input(|i| i.time), frame.info().cpu_usage);

        self.handle_ui_cmd();

        if self
            .main_state
            .selected_pattern()
            .as_ref()
            .is_some_and(|ptr| ptr.upgrade().is_none())
        {
            self.main_state.select_pattern(None);
            self.sheet_reader.set_context(SheetContext::Track);
        }

        self.draw_ui(ctx);
        self.draw_active_tool_windows(ctx);
        self.error_modal.try_draw(ctx);
        ctx.request_repaint(); // Uncomment this for continuous repainting (fix some UI update issues)
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

        egui::SidePanel::left(WidgetId::MainAppLeftExplorerPanel).show(ctx, |ui| self.explorer(ui));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Tracks");
        });
    }

    fn draw_active_tool_windows(&mut self, ctx: &egui::Context) {
        for tool in self.tools.iter_mut() {
            if tool.window_open() {
                tool.draw(ctx);
            }
        }
    }

    // TODO: design & impl the actual menus
    fn app_menu(&mut self, ui: &mut egui::Ui) {
        ui.label("menubar");
    }

    fn context_control(&mut self, ui: &mut egui::Ui) {
        // context swicher
        egui::Frame::NONE.show(ui, |ui| {
            ui.spacing_mut().item_spacing = emath::vec2(0., 0.);
            if ui
                .add(
                    egui::Button::new("󰲸 ")
                        .selected(self.sheet_reader.context_is_track())
                        .corner_radius(egui::CornerRadius {
                            ne: 0,
                            se: 0,
                            ..ui.style().noninteractive().corner_radius
                        })
                        .frame_when_inactive(true),
                )
                .clicked()
            {
                self.sheet_reader.set_context(SheetContext::Track);
            }

            {
                let selected_pattern = self.main_state.selected_pattern();

                if ui
                    .add_enabled(
                        selected_pattern.is_some(),
                        egui::Button::new(format!(
                            "󰎅  {}",
                            selected_pattern
                                .as_ref()
                                .map(|ptr| ptr
                                    .upgrade()
                                    .map(|pat| pat.read().name.clone())
                                    .unwrap_or_default())
                                .unwrap_or_default()
                        ))
                        .corner_radius(egui::CornerRadius {
                            nw: 0,
                            sw: 0,
                            ..ui.style().noninteractive().corner_radius
                        })
                        .selected(!self.sheet_reader.context_is_track())
                        .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    self.sheet_reader.set_context(SheetContext::Pattern(
                        selected_pattern.as_ref().unwrap().clone(),
                    ));
                }
            }
        });

        // play/pause control
        let playing = self.metronome.playing();
        if ui
            .add(
                egui::Button::new(if playing { " " } else { " " })
                    .selected(playing)
                    .frame_when_inactive(true),
            )
            .clicked()
        {
            self.metronome.toggle_playing(None);
        }

        // stop control
        if ui
            .add_enabled(!self.metronome.stopped(), egui::Button::new(""))
            .clicked()
        {
            self.metronome.stop();
        };

        // bpm control
        ui.add(
            egui::DragValue::new(self.metronome.bpm_mut().deref_mut())
                .range(1..=640)
                .prefix("BPM "),
        );

        // TODO: impl actual context progress bar
        ui.label(format!(
            " {}/{:?}",
            self.metronome.query_tick(),
            self.metronome.top_tick()
        ));
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        for tool in self.tools.iter_mut() {
            if ui
                .add(
                    egui::Button::new(tool.icon())
                        .selected(tool.window_open())
                        .frame_when_inactive(true),
                )
                .clicked()
            {
                tool.toggle_open(None);
            }
        }
    }

    fn explorer(&mut self, ui: &mut egui::Ui) {
        for pattern in self.sheet_reader.patterns().deref() {
            if ui
                .add(
                    egui::Button::new(&pattern.read().name)
                        .selected(
                            self.main_state
                                .selected_pattern()
                                .as_ref()
                                .is_some_and(|ptr| {
                                    ptr.upgrade()
                                        .is_some_and(|pat| pat.read().name == pattern.read().name)
                                }),
                        )
                        .frame_when_inactive(true),
                )
                .clicked()
            {
                let pat_ptr = Arc::downgrade(pattern);
                self.main_state.select_pattern(Some(pat_ptr.clone()));
                if !self.sheet_reader.context_is_track() {
                    self.sheet_reader
                        .set_context(SheetContext::Pattern(pat_ptr));
                }
            };
        }
    }
}

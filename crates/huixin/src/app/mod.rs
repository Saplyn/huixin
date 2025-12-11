use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Weak, mpsc},
    thread,
};

use parking_lot::RwLock;

use crate::{
    app::{
        helpers::WidgetId,
        tools::{ToolWindow, pattern_editor::PatternEditor, tester::Tester},
        widgets::{error_modal::ErrorModal, performance::Performance},
    },
    routines::{
        instructor::Instructor,
        metronome::{Metronome, TICK_PER_BEAT},
        sheet_reader::SheetReader,
    },
    sheet::pattern::{
        SheetPattern, SheetPatternTrait, SheetPatternType,
        midi::{MidiNote, MidiPattern},
    },
};

mod helpers;
mod tools;
mod widgets;

// LYN: Main App State Holder

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PlayerContext {
    #[default]
    Sheet,
    Pattern,
}

#[derive(Debug)]
pub struct CommonState {
    selected_pattern: RwLock<Option<Weak<RwLock<SheetPattern>>>>,
    player_context: RwLock<PlayerContext>,
}

impl CommonState {
    pub fn selected_pattern(&self) -> Option<Arc<RwLock<SheetPattern>>> {
        self.selected_pattern.read().clone()?.upgrade()
    }
    pub fn select_pattern(&self, pattern: Option<Weak<RwLock<SheetPattern>>>) {
        *self.selected_pattern.write() = pattern;
    }

    pub fn set_context(&self, context: PlayerContext) {
        *self.player_context.write() = context;
    }
    pub fn player_context(&self) -> PlayerContext {
        *self.player_context.read()
    }

    /// Returns the tick limit for metronome.
    pub fn metro_tick_limit(&self) -> Option<u64> {
        match *self.player_context.read() {
            PlayerContext::Sheet => None,
            PlayerContext::Pattern => self
                .selected_pattern
                .read()
                .as_ref()
                .and_then(|ptr| ptr.upgrade())
                .map(|pat| pat.read().beats() * TICK_PER_BEAT - 1),
        }
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
    common: Arc<CommonState>,
    metronome: Arc<Metronome>,
    sheet_reader: Arc<SheetReader>,
    instructor: Arc<Instructor>,
}

impl MainApp {
    pub fn prepare() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();

        let common = Arc::new(CommonState {
            selected_pattern: RwLock::new(None),
            player_context: RwLock::new(PlayerContext::Sheet),
        });
        let metronome = Arc::new(Metronome::init());
        let sheet_reader = Arc::new(SheetReader::init());
        let instructor = Arc::new(Instructor::init());

        let tools: Vec<Box<dyn ToolWindow>> = vec![
            Box::new(Tester::new(
                common.clone(),
                metronome.clone(),
                sheet_reader.clone(),
            )),
            Box::new(PatternEditor::new(
                common.clone(),
                metronome.clone(),
                sheet_reader.clone(),
            )),
        ];

        // FIXME: test data
        sheet_reader.add_pattern("Test 1".to_string(), SheetPatternType::Midi);
        sheet_reader.add_pattern("Test 2".to_string(), SheetPatternType::Midi);
        sheet_reader.add_pattern("Test 3".to_string(), SheetPatternType::Midi);
        *sheet_reader.patterns_mut().deref_mut()[0].write() = {
            let mut pat = MidiPattern::new("Test 1".to_string(), None);
            pat.beats = 4;
            let tmp = TICK_PER_BEAT;
            pat.add_note(MidiNote::new(61, u16::MAX, tmp, tmp));
            pat.add_note(MidiNote::new(60, u16::MAX, 0, tmp));
            pat.add_note(MidiNote::new(63, u16::MAX, 3 * tmp, tmp));
            pat.add_note(MidiNote::new(62, u16::MAX, 2 * tmp, tmp));

            SheetPattern::Midi(pat)
        };
        // ===

        thread::spawn({
            let state = metronome.clone();
            let common = common.clone();
            let cmd_tx = cmd_tx.clone();
            move || Metronome::main(state, common, cmd_tx)
        });
        thread::spawn({
            let state = sheet_reader.clone();
            let common = common.clone();
            let metro = metronome.clone();
            let cmd_tx = cmd_tx.clone();
            move || SheetReader::main(state, common, metro, cmd_tx)
        });
        thread::spawn({
            let state = instructor.clone();
            let cmd_tx = cmd_tx.clone();
            move || Instructor::main(state, cmd_tx)
        });

        Self {
            cmd_tx,
            cmd_rx,
            performance: Default::default(),
            error_modal: Default::default(),
            tools,
            common,
            metronome,
            sheet_reader,
            instructor,
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

        if self.common.selected_pattern().is_none() {
            self.common.select_pattern(None);
            self.common.set_context(PlayerContext::Sheet);
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
        // context switcher
        egui::Frame::NONE.show(ui, |ui| {
            ui.spacing_mut().item_spacing = emath::vec2(0., 0.);
            if ui
                .add(
                    egui::Button::new("󰲸 ")
                        .selected(self.common.player_context() == PlayerContext::Sheet)
                        .corner_radius(egui::CornerRadius {
                            ne: 0,
                            se: 0,
                            ..ui.style().noninteractive().corner_radius
                        })
                        .frame_when_inactive(true),
                )
                .clicked()
            {
                self.common.set_context(PlayerContext::Sheet);
            }

            {
                let selected_pattern = self.common.selected_pattern();

                if ui
                    .add_enabled(
                        selected_pattern.is_some(),
                        egui::Button::new(format!(
                            "󰎅  {}",
                            selected_pattern
                                .map(|pat| pat.read().name_ref().to_owned())
                                .unwrap_or_default()
                        ))
                        .corner_radius(egui::CornerRadius {
                            nw: 0,
                            sw: 0,
                            ..ui.style().noninteractive().corner_radius
                        })
                        .selected(self.common.player_context() == PlayerContext::Pattern)
                        .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    self.common.set_context(PlayerContext::Pattern);
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
            self.common.metro_tick_limit()
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
                    egui::Button::new(format!(
                        "{} {}",
                        pattern.read().icon_ref(),
                        pattern.read().name_ref()
                    ))
                    .selected(
                        self.common
                            .selected_pattern()
                            .is_some_and(|pat| pat.read().name_ref() == pattern.read().name_ref()),
                    )
                    .frame_when_inactive(true),
                )
                .clicked()
            {
                let pat_ptr = Arc::downgrade(pattern);
                self.common.select_pattern(Some(pat_ptr.clone()));
            };
        }
    }
}

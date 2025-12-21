use std::{
    fs,
    ops::DerefMut,
    sync::{Arc, mpsc},
    thread,
};

use egui::containers::menu::MenuButton;
use log::warn;
use parking_lot::{RwLock, RwLockReadGuard};

use self::{
    helpers::WidgetId,
    tools::{
        ToolWindow, connection_manager::ConnectionManager, pattern_editor::PatternEditor,
        tester::Tester,
    },
    widgets::{error_modal::ErrorModal, performance::Performance},
};
use crate::{
    APP_ID,
    app::{
        persistence::{PersistedState, WorkingDirectory, form_persistable},
        tools::ToolWindowId,
    },
    model::pattern::{SheetPattern, SheetPatternTrait, SheetPatternType},
    routines::{
        RoutineId,
        guardian::Guardian,
        instructor::Instructor,
        metronome::{Metronome, TICK_PER_BEAT},
        sheet_reader::SheetReader,
    },
};

mod helpers;
pub(crate) mod persistence;
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
    pub err_modal_message: RwLock<Option<String>>,
    selected_pattern: RwLock<Option<String>>,
    player_context: RwLock<PlayerContext>,
}

impl CommonState {
    pub fn selected_pattern_id(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.selected_pattern.read()
    }
    pub fn selected_pattern(
        &self,
        sheet_reader: Arc<SheetReader>,
    ) -> Option<Arc<RwLock<SheetPattern>>> {
        sheet_reader.get_pattern(self.selected_pattern.read().as_ref()?)
    }
    pub fn select_pattern(&self, opt_id: Option<String>) {
        *self.selected_pattern.write() = opt_id;
    }

    pub fn set_context(&self, context: PlayerContext) {
        *self.player_context.write() = context;
    }
    pub fn player_context(&self) -> PlayerContext {
        *self.player_context.read()
    }

    /// Returns the tick limit for metronome.
    pub fn metro_tick_limit(&self, sheet_reader: Arc<SheetReader>) -> Option<u64> {
        match *self.player_context.read() {
            PlayerContext::Sheet => None,
            PlayerContext::Pattern => self
                .selected_pattern(sheet_reader)
                .as_ref()
                .map(|pat| pat.read().beats() * TICK_PER_BEAT - 1),
        }
    }
}

#[derive(Debug)]
pub struct MainApp {
    // app states
    working_directory: Option<WorkingDirectory>,

    // widget states
    performance: Performance,
    tools: Vec<Box<dyn ToolWindow>>,

    // routine states
    common: Arc<CommonState>,
    metronome: Arc<Metronome>,
    sheet_reader: Arc<SheetReader>,
    instructor: Arc<Instructor>,
}

impl MainApp {
    pub fn init() -> Self {
        let (msg_tx, msg_rx) = mpsc::channel();

        let common = Arc::new(CommonState {
            err_modal_message: RwLock::new(None),
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
                instructor.clone(),
            )),
            Box::new(ConnectionManager::new(common.clone(), instructor.clone())),
        ];

        let routines = vec![
            (
                RoutineId::Metronome,
                thread::spawn({
                    let state = metronome.clone();
                    let common = common.clone();
                    let sheet_reader = sheet_reader.clone();
                    move || Metronome::main(state, common, sheet_reader)
                }),
            ),
            (
                RoutineId::SheetReader,
                thread::spawn({
                    let state = sheet_reader.clone();
                    let common = common.clone();
                    let metro = metronome.clone();
                    move || SheetReader::main(state, common, metro, msg_tx)
                }),
            ),
            (
                RoutineId::Instructor,
                thread::spawn({
                    let state = instructor.clone();
                    move || Instructor::main(state, msg_rx)
                }),
            ),
        ];
        thread::spawn({
            let common = common.clone();
            move || Guardian::main(routines, common)
        });

        Self {
            working_directory: None,
            performance: Default::default(),
            tools,
            common,
            metronome,
            sheet_reader,
            instructor,
        }
    }
    fn persist_state(&mut self) {
        let Some(cwd) = self.working_directory.as_ref() else {
            return;
        };

        let state = form_persistable(
            self.common.clone(),
            self.metronome.clone(),
            self.sheet_reader.clone(),
            self.instructor.clone(),
        );
        self.instructor.spawn({
            let file = cwd.state_path(APP_ID);
            move || {
                let file_content = json::to_string_pretty(&state).unwrap();
                if let Err(e) = fs::create_dir_all(file.parent().unwrap()) {
                    warn!("Failed to create directories for state persistence: {}", e);
                    return;
                }
                if let Err(e) = fs::write(&file, file_content) {
                    warn!("Failed to persist state to file {:?}: {}", file, e);
                }
            }
        });
    }
    fn restore_state(&mut self, state: PersistedState) {
        if let Some(cwd) = self.working_directory.as_ref() {}
        // TODO:
        let PersistedState {
            bpm,
            patterns,
            tracks,
            targets,
        } = state;

        self.metronome.restore_state(bpm);
        self.sheet_reader.restore_state(patterns, tracks);
        self.instructor.restore_state(targets);
    }

    const STORAGE_KEY_CWD: &str = "lyn:working-directory";
    pub fn prepare_launch(&mut self, cc: &eframe::CreationContext<'_>) {
        if let Some(storage) = cc.storage {
            self.working_directory =
                eframe::get_value(storage, Self::STORAGE_KEY_CWD).unwrap_or_default();

            if let Some(cwd) = self.working_directory.as_ref() {
                let state_file = cwd.state_path(APP_ID);
                if state_file.exists() {
                    match fs::read_to_string(&state_file).and_then(|content| {
                        json::from_str::<PersistedState>(&content).map_err(|e| e.into())
                    }) {
                        Ok(state) => {
                            self.restore_state(state);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to restore persisted state from file {:?}: {}",
                                state_file, e
                            );
                        }
                    }
                }
            }
        }
    }
}

// LYN: Main App UI Implementation

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.performance
            .update_frame_history(ctx.input(|i| i.time), frame.info().cpu_usage);

        if self
            .common
            .selected_pattern(self.sheet_reader.clone())
            .is_none()
        {
            self.common.select_pattern(None);
            self.common.set_context(PlayerContext::Sheet);
        }

        if self.working_directory.is_none() {
            self.draw_placeholder_ui(ctx);
        } else {
            self.draw_studio_ui(ctx);
            self.draw_active_tool_windows(ctx);
        }

        if let Some(msg) = self.common.err_modal_message.read().as_ref() {
            ErrorModal::new(msg).draw(ctx);
        }
        ctx.request_repaint(); // Uncomment this for continuous repainting (fix some UI update issues)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Self::STORAGE_KEY_CWD, &self.working_directory);
    }
}

impl MainApp {
    fn draw_placeholder_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 12.0);
                if ui
                    .button(egui::RichText::new("选择工作目录").heading())
                    .clicked()
                {
                    // FIXME:
                    self.working_directory = rfd::FileDialog::new().pick_folder().map(|p| p.into());
                }
            });
        });
    }

    fn draw_studio_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top(WidgetId::MainAppTopToolBar).show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .inner_margin(emath::vec2(0., 6.))
                    .show(ui, |ui| {
                        self.app_menu(ui);
                    });
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
            .min_width(200.)
            .max_width(200.)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .inner_margin(emath::vec2(0., 4.))
                    .show(ui, |ui| {
                        ui.style_mut().spacing.item_spacing = emath::vec2(0., 4.);
                        self.explorer(ui);
                    });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                self.track_editor(ui);
            });
    }

    fn draw_active_tool_windows(&mut self, ctx: &egui::Context) {
        for tool in self.tools.iter_mut() {
            if tool.window_open() {
                tool.draw(ctx);
            }
        }
    }

    fn app_menu(&mut self, ui: &mut egui::Ui) {
        MenuButton::from_button(egui::Button::new("󰍜 ").frame_when_inactive(false)).ui(ui, |ui| {
            if ui.button("保存").clicked() {
                self.persist_state();
                ui.close();
            }
        });
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
                let selected_pattern = self.common.selected_pattern(self.sheet_reader.clone());

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
            self.common.metro_tick_limit(self.sheet_reader.clone())
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
        if ui
            .add_sized(
                [ui.available_width(), 30.],
                egui::Button::new(egui::RichText::new("添加片段")),
            )
            .clicked()
        {
            self.sheet_reader.add_pattern(SheetPatternType::Midi);
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_be_removed = Vec::new();
            for entry in self.sheet_reader.patterns_iter() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.style_mut().spacing.item_spacing = emath::vec2(4., 0.);
                    if ui.button(" ").clicked() {
                        to_be_removed.push(entry.key().clone());
                    }

                    let pat_button = ui.add_sized(
                        [ui.available_width(), 0.],
                        egui::Button::new(format!(
                            "{} {}",
                            entry.value().read().icon_ref(),
                            entry.value().read().name_ref()
                        ))
                        .right_text("")
                        .selected(
                            self.common
                                .selected_pattern_id()
                                .as_ref()
                                .is_some_and(|pat| pat == entry.key()),
                        )
                        .frame_when_inactive(true),
                    );
                    if pat_button.clicked() {
                        self.common.select_pattern(Some(entry.key().clone()));
                    };
                    if pat_button.double_clicked() {
                        *self
                            .tools
                            .iter_mut()
                            .find(|tool| tool.tool_id() == ToolWindowId::PatternEditor)
                            .unwrap()
                            .window_open_mut() = true;
                    };
                });
            }
            for pat_id in to_be_removed {
                self.sheet_reader.del_pattern(&pat_id);
            }
        });
    }

    fn track_editor(&mut self, ui: &mut egui::Ui) {
        ui.label("Tracks");
    }
}

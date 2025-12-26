use std::{
    fs,
    ops::DerefMut,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use egui::containers::menu::MenuButton;
use log::warn;

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
    app::{helpers::text_color, tools::ToolWindowId, widgets::track_editor::TrackEditor},
    model::{
        pattern::{SheetPatternTrait, SheetPatternType},
        persistence::{AppStorage, WorkingDirectory},
        state::CentralState,
        track::SheetTrackType,
    },
    routines::{RoutineId, guardian, instructor, metronome, sheet_reader},
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
pub struct MainApp {
    // app states
    working_directory: Option<WorkingDirectory>,

    // widget states
    performance: Performance,
    tools: Vec<Box<dyn ToolWindow>>,
    track_editor: TrackEditor,

    // central state
    state: Arc<CentralState>,
}

impl MainApp {
    pub fn init() -> Self {
        let (msg_tx, msg_rx) = mpsc::channel();
        let state = Arc::new(CentralState::init());

        let tools: Vec<Box<dyn ToolWindow>> = vec![
            Box::new(Tester::new(state.clone())),
            Box::new(PatternEditor::new(state.clone())),
            Box::new(ConnectionManager::new(state.clone())),
        ];

        let routines = vec![
            (
                RoutineId::Metronome,
                thread::spawn({
                    let state = state.clone();
                    move || metronome::main(state)
                }),
            ),
            (
                RoutineId::SheetReader,
                thread::spawn({
                    let state = state.clone();
                    move || sheet_reader::main(state, msg_tx)
                }),
            ),
            (
                RoutineId::Instructor,
                thread::spawn({
                    let state = state.clone();
                    move || instructor::main(state, msg_rx)
                }),
            ),
        ];
        thread::spawn({
            let state = state.clone();
            move || guardian::main(state, routines)
        });

        Self {
            working_directory: None,
            performance: Default::default(),
            track_editor: TrackEditor::new(state.clone()),
            tools,
            state,
        }
    }
    fn persist_sheet(&self) {
        let Some(cwd) = self.working_directory.as_ref() else {
            return;
        };

        self.state.worker_spawn_task({
            let file_content = self.state.sheet_to_json_string_pretty().unwrap();
            let file = cwd.state_path(APP_ID);
            move || {
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
    fn restore_sheet(&self) {
        // read the file, use `self.state.sheet_from_json_str` to restore the state
        let Some(cwd) = self.working_directory.as_ref() else {
            return;
        };
        let state_file = cwd.state_path(APP_ID);
        if state_file.exists() {
            match fs::read_to_string(&state_file) {
                Ok(str) => {
                    if let Err(e) = self.state.sheet_from_json_str(&str) {
                        warn!(
                            "Failed to restore persisted state from file {:?}: {}",
                            state_file, e
                        );
                    };
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

    const STORAGE_KEY_CWD: &str = "working-directory";
    pub fn prepare_launch(&mut self, cc: &eframe::CreationContext<'_>) {
        let Some(storage) = cc.storage else {
            return;
        };

        self.working_directory =
            eframe::get_value(storage, &AppStorage::key(Self::STORAGE_KEY_CWD)).unwrap_or_default();

        self.restore_sheet();

        for tool in self.tools.iter_mut() {
            let key = AppStorage::key(tool.tool_id().to_string());
            let open = eframe::get_value(storage, &key).unwrap_or_default();
            *tool.window_open_mut() = open;
        }
    }
}

// LYN: Main App UI Implementation

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.performance
            .update_frame_history(ctx.input(|i| i.time), frame.info().cpu_usage);

        if self.state.selected_pattern().is_none() {
            self.state.select_pattern(None);
            self.state.player_set_context(PlayerContext::Sheet);
        }

        if self.working_directory.is_none() {
            self.draw_placeholder_ui(ctx);
        } else {
            self.draw_studio_ui(ctx);
            self.draw_active_tool_windows(ctx);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::S) && (i.modifiers.ctrl || i.modifiers.command)) {
            self.persist_sheet();
        }

        if let Some(msg) = self.state.get_err_msg().as_ref() {
            ErrorModal::new(msg).draw(ctx);
        }
        ctx.request_repaint(); // Uncomment this for continuous repainting (fix some UI update issues)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Self::STORAGE_KEY_CWD, &self.working_directory);
        for tool in self.tools.iter() {
            eframe::set_value(
                storage,
                &AppStorage::key(tool.tool_id().to_string()),
                &tool.window_open(),
            );
        }
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(5)
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
            .min_width(240.)
            .max_width(240.)
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
                self.track_editor.show(ui);
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
                self.persist_sheet();
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
                        .selected(self.state.player_context() == PlayerContext::Sheet)
                        .corner_radius(egui::CornerRadius {
                            ne: 0,
                            se: 0,
                            ..ui.style().noninteractive().corner_radius
                        })
                        .frame_when_inactive(true),
                )
                .clicked()
            {
                self.state.player_set_context(PlayerContext::Sheet);
            }

            {
                let selected_pattern = self.state.selected_pattern();

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
                        .selected(self.state.player_context() == PlayerContext::Pattern)
                        .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    self.state.player_set_context(PlayerContext::Pattern);
                }
            }
        });

        // play/pause control
        let playing = self.state.metro_playing();
        if ui
            .add(
                egui::Button::new(if playing { " " } else { " " })
                    .selected(playing)
                    .frame_when_inactive(true),
            )
            .clicked()
        {
            self.state.metro_toggle_playing(None);
        }

        // stop control
        if ui
            .add_enabled(!self.state.metro_stopped(), egui::Button::new(""))
            .clicked()
        {
            self.state.metro_make_stop();
        };

        // bpm control
        ui.add(
            egui::DragValue::new(self.state.sheet_bpm_mut().deref_mut())
                .range(1..=640)
                .prefix("BPM "),
        );

        // TODO: impl actual context progress bar
        let limit = self.state.metro_tick_limit();
        ui.add(
            egui::DragValue::new(self.state.sheet_length_in_beats_mut().deref_mut())
                .range(1..=u64::MAX)
                .prefix("Beats "),
        );
        ui.add(
            egui::Slider::new(self.state.metro_tick_mut().deref_mut(), 0..=limit)
                .suffix(format!("/{limit}")),
        );
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
            .add_sized([ui.available_width(), 30.], egui::Button::new("添加片段"))
            .clicked()
        {
            self.state.sheet_add_pattern(SheetPatternType::Midi);
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_be_removed = Vec::new();
            for entry in self.state.sheet_patterns_iter() {
                let guard = entry.value().read();
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing = emath::vec2(4., 0.);
                    let pat_color = guard.color();
                    let icon_button = ui.add_sized(
                        [46., ui.available_height()],
                        egui::Button::new(
                            egui::RichText::new(guard.icon_ref())
                                .heading()
                                .color(pat_color.lerp_to_gamma(text_color(pat_color), 0.6)),
                        )
                        .fill(pat_color),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button(egui::RichText::new(" ").heading()).clicked() {
                            to_be_removed.push(entry.key().clone());
                        }

                        let pat_button = ui.add_sized(
                            ui.available_size(),
                            egui::Button::new(guard.name_ref())
                                .right_text("")
                                .selected(
                                    self.state
                                        .selected_pattern_id()
                                        .as_ref()
                                        .is_some_and(|pat| pat == entry.key()),
                                )
                                .frame_when_inactive(true),
                        );
                        if pat_button.clicked() || icon_button.clicked() {
                            self.state.select_pattern(Some(entry.key().clone()));
                        };
                        if pat_button.double_clicked() || icon_button.double_clicked() {
                            *self
                                .tools
                                .iter_mut()
                                .find(|tool| tool.tool_id() == ToolWindowId::PatternEditor)
                                .unwrap()
                                .window_open_mut() = true;
                        };
                    });
                });
            }
            for pat_id in to_be_removed {
                self.state.sheet_del_pattern(&pat_id);
            }
        });
    }
}

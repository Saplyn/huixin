use std::{
    fs,
    ops::DerefMut,
    path::PathBuf,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use egui::containers::menu::MenuButton;
use egui_dnd::dnd;
use log::{info, warn};
use lyn_util::{egui::text_color, persist::AppStore};

use self::{
    helpers::WidgetId,
    tools::{
        ToolWindow, connection_manager::ConnectionManager, pattern_editor::PatternEditor,
        tester::Tester,
    },
    widgets::{error_modal::ErrorModal, performance::Performance},
};
use crate::{
    app::{tools::ToolWindowId, widgets::track_editor::TrackEditor},
    model::{
        pattern::{SheetPatternTrait, SheetPatternType},
        state::{CentralState, UiState},
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
            performance: Default::default(),
            track_editor: TrackEditor::new(state.clone()),
            tools,
            state,
        }
    }

    const STORAGE_KEY_PROJECT_DIR: &str = "project-directory";
    pub fn prepare_launch(&mut self, cc: &eframe::CreationContext<'_>) {
        let egui_ctx = cc.egui_ctx.clone();
        self.state.ui.ctx.set(egui_ctx).unwrap();

        let Some(storage) = cc.storage else {
            return;
        };

        let dir: Option<PathBuf> =
            eframe::get_value(storage, &AppStore::key(Self::STORAGE_KEY_PROJECT_DIR))
                .unwrap_or_default();
        if let Some(dir) = dir {
            self.state.load_project(dir);
        }

        for tool in self.tools.iter_mut() {
            let key = AppStore::key(tool.tool_id().to_store_key());
            let open = eframe::get_value(storage, &key).unwrap_or_default();
            *tool.window_open_mut() = open;
        }
        *self.state.ui.track_editor_size_per_beat.write() =
            eframe::get_value(storage, &AppStore::key(UiState::STORAGE_KEY_TRACK_SPB))
                .unwrap_or(UiState::MIN_SIZE_PER_BEAT);
        *self.state.ui.pattern_editor_size_per_beat.write() =
            eframe::get_value(storage, &AppStore::key(UiState::STORAGE_KEY_PATTERN_SPB))
                .unwrap_or(UiState::MIN_SIZE_PER_BEAT);
    }

    // (safe to call if `working_directory` is `None`)
    fn persist_sheet_blocking(&self) -> Result<(), ()> {
        let project_guard = self.state.get_project();
        let project = project_guard.as_ref().ok_or(())?;

        let sheet_file_content = self.state.sheet_to_ron_string_pretty().map_err(|_| ())?;
        let sheet_file_path = project.sheet_file();

        if let Some(parent) = sheet_file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                warn!("Failed to create directories for sheet persistence: {}", e);
            })?;
        }
        fs::write(&sheet_file_path, sheet_file_content).map_err(|e| {
            warn!(
                "Failed to persist sheet to file {:?}: {}",
                sheet_file_path, e
            );
        })?;

        Ok(())
    }
    // (safe to call if `working_directory` is `None`)
    fn restore_sheet_blocking(&self) -> Result<(), ()> {
        let project_guard = self.state.get_project();
        let project = project_guard.as_ref().ok_or(())?;

        let state_file = project.sheet_file();
        if !state_file.exists() {
            warn!(
                "No persisted sheet file found at {:?}, performing an empty save.",
                state_file
            );
            self.persist_sheet_blocking()?;
            return Err(());
        }
        let str = fs::read_to_string(&state_file).map_err(|e| {
            warn!(
                "Failed to read persisted sheet from file {:?}: {}",
                state_file, e
            );
        })?;
        self.state.sheet_from_ron_str(&str).map_err(|e| {
            warn!(
                "Failed to restore persisted state from file {:?}: {}",
                state_file, e
            );
        })?;

        Ok(())
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

        if self.state.get_project().is_none() {
            self.draw_project_selector_ui(ctx);
        } else {
            if !self.state.sheet_loaded() {
                self.restore_sheet_blocking();
            }
            self.draw_studio_ui(ctx);
            self.draw_active_tool_windows(ctx);
        }

        self.handle_keyboard_shortcuts(ctx);

        if let Some(msg) = self.state.app_get_err_msg().as_ref() {
            ErrorModal::new(msg).draw(ctx);
        }
        // ctx.request_repaint(); // Uncomment this for continuous repainting (fix some UI update issues)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(
            storage,
            &AppStore::key(Self::STORAGE_KEY_PROJECT_DIR),
            &self
                .state
                .get_project()
                .as_ref()
                .map(|project| &project.project_dir),
        );
        for tool in self.tools.iter() {
            eframe::set_value(
                storage,
                &AppStore::key(tool.tool_id().to_store_key()),
                &tool.window_open(),
            );
        }
        eframe::set_value(
            storage,
            &AppStore::key(UiState::STORAGE_KEY_TRACK_SPB),
            &self.state.ui.track_editor_size_per_beat,
        );
        eframe::set_value(
            storage,
            &AppStore::key(UiState::STORAGE_KEY_PATTERN_SPB),
            &self.state.ui.pattern_editor_size_per_beat,
        );
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(5)
    }
}

impl MainApp {
    fn draw_project_selector_ui(&mut self, ctx: &egui::Context) {
        fn select_working_dir(state: &CentralState) {
            struct ScopeGuard<'state> {
                state: &'state CentralState,
            }
            impl<'state> Drop for ScopeGuard<'state> {
                fn drop(&mut self) {
                    *self.state.ui.selecting_project_dir.write() = false;
                }
            }

            let _guard = ScopeGuard { state };
            let Some(project_dir) = rfd::FileDialog::new().pick_folder() else {
                return;
            };
            if let Err(err) = state.load_project(project_dir) {
                warn!("Failed to load project: {}", err);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if *self.state.ui.selecting_project_dir.read() {
                ui.disable();
            }

            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 12.0);
                if ui
                    .button(egui::RichText::new("选择工作目录").heading())
                    .clicked()
                {
                    *self.state.ui.selecting_project_dir.write() = true;
                    ctx.request_repaint();
                    self.state.worker_spawn_task({
                        let state = self.state.clone();
                        move || select_working_dir(&state)
                    });
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
            ui.menu_button("项目", |ui| {
                if ui.button("保存").clicked() {
                    self.persist_sheet_blocking();
                    ui.close();
                }
                if ui.button("关闭").clicked() {
                    self.state.close_project();
                    ui.close();
                }
            });
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

        // context progress bar
        let limit = self.state.metro_tick_limit();
        ui.add(
            egui::DragValue::new(self.state.sheet_length_in_beats_mut().deref_mut())
                .range(self.state.sheet_min_length_in_beats()..=u64::MAX)
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
            dnd(ui, WidgetId::MainAppExplorerPatternsOrderingDnd).show_vec(
                &mut self.state.sheet_patterns_ordering_mut(),
                |ui, pat_id, handle, _state| {
                    let Some(arc) = self.state.sheet_get_pattern(pat_id) else {
                        return;
                    };
                    let guard = arc.read();
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing = emath::vec2(4., 0.);
                        let pat_color = guard.color();

                        handle.ui(ui, |ui| {
                            ui.add_sized(
                                [46., ui.available_height()],
                                egui::Button::new(
                                    egui::RichText::new(guard.icon_ref())
                                        .heading()
                                        .color(pat_color.lerp_to_gamma(text_color(pat_color), 0.6)),
                                )
                                .fill(pat_color),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button(egui::RichText::new(" ").heading()).clicked() {
                                to_be_removed.push(pat_id.clone());
                            }

                            let pat_button = ui.add_sized(
                                ui.available_size(),
                                egui::Button::new(guard.name_ref())
                                    .right_text("")
                                    .selected(
                                        self.state
                                            .selected_pattern_id()
                                            .as_ref()
                                            .is_some_and(|pat| pat == pat_id),
                                    )
                                    .frame_when_inactive(true),
                            );
                            if pat_button.clicked() {
                                self.state.select_pattern(Some(pat_id.clone()));
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
                    });
                },
            );
            for pat_id in to_be_removed {
                self.state.sheet_del_pattern(&pat_id);
            }
        });
    }
}

impl MainApp {
    fn handle_keyboard_shortcuts(&self, ctx: &egui::Context) {
        // `ctrl+s`: save sheet
        if ctx.input(|i| i.key_pressed(egui::Key::S) && (i.modifiers.ctrl || i.modifiers.command)) {
            self.persist_sheet_blocking();
        }
    }
}

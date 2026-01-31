use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use egui::{TextBuffer, containers::menu::MenuButton};
use egui_dnd::dnd;
use egui_snarl::ui::{PinPlacement, SnarlStyle, SnarlWidget};
use log::warn;
use lyn_util::{egui::text_color, persist::AppStore};
use parking_lot::RwLock;

use crate::{
    APP_ID,
    app::{patch_viewer::PatchViewerOutput, widgets::error_modal::ErrorModal},
    model::{
        patch::Patch,
        state::{CentralState, PersistedState},
    },
    routines::{RoutineId, guardian, listener, processor},
};

use self::{helpers::WidgetId, patch_viewer::PatchViewer, widgets::performance::Performance};

mod helpers;
mod patch_viewer;
mod widgets;

// LYN: Main App State Holder

#[derive(Debug)]
pub struct MainApp {
    // local state
    new_patch_name: String,

    // widget states
    performance: Performance,

    // central state
    processor_cmd_tx: mpsc::Sender<processor::Command>,
    state: Arc<CentralState>,
}

impl MainApp {
    pub fn init() -> Self {
        let (processor_cmd_tx, processor_cmd_rx) = mpsc::channel();
        let state = Arc::new(CentralState::init());

        let routines = vec![
            (
                RoutineId::Processor,
                thread::spawn({
                    let state = state.clone();
                    move || processor::main(state, processor_cmd_rx)
                }),
            ),
            (
                RoutineId::Listener,
                thread::spawn({
                    let state = state.clone();
                    move || listener::main(state)
                }),
            ),
        ];
        thread::spawn({
            let state = state.clone();
            move || guardian::main(state, routines)
        });

        Self {
            new_patch_name: String::new(),
            performance: Default::default(),
            processor_cmd_tx,
            state,
        }
    }

    const STORAGE_KEY_PROJECT_DIR: &str = "project-directory";
    pub fn prepare_launch(&self, cc: &eframe::CreationContext<'_>) {
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
    }

    // (safe to call if `working_directory` is `None`)
    fn persist_sheet_blocking(&self) -> Result<(), ()> {
        let project_guard = self.state.get_project();
        let project = project_guard.as_ref().ok_or(())?;

        let (patches, state) = self.state.sheet_to_persisted();
        fs::create_dir_all(project.patch_dir()).map_err(|e| {
            warn!("Failed to create directories for sheet persistence: {}", e);
        })?;
        for (name, patch) in patches {
            let patch_file_path = project.patch_file(&name);
            let patch_file_content =
                ron::ser::to_string_pretty(&patch, ron::ser::PrettyConfig::default()).map_err(
                    |e| {
                        warn!("Failed to serialize patch for sheet persistence: {}", e);
                    },
                )?;

            fs::write(&patch_file_path, patch_file_content).map_err(|e| {
                warn!("Failed to write patch file for sheet persistence: {}", e);
            })?;
        }

        let state_file_path = project.state_file(APP_ID);
        let state_file_content =
            ron::ser::to_string_pretty(&state, ron::ser::PrettyConfig::default())
                .map_err(|_| ())?;
        if let Some(parent) = state_file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                warn!("Failed to create directories for sheet persistence: {}", e);
            })?;
        }
        fs::write(&state_file_path, state_file_content).map_err(|e| {
            warn!("Failed to write state file for sheet persistence: {}", e);
        })?;

        Ok(())
    }
    // (safe to call if `working_directory` is `None`)
    fn restore_sheet_blocking(&self) -> Result<(), ()> {
        let project_guard = self.state.get_project();
        let project = project_guard.as_ref().ok_or(())?;

        // Load all patches from patches/ directory
        let mut patches = HashMap::new();
        let patch_dir = project.patch_dir();

        if patch_dir.exists()
            && patch_dir.is_dir()
            && let Ok(entries) = fs::read_dir(&patch_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                    let patch_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string());

                    if let Some(name) = patch_name {
                        match fs::read_to_string(&path) {
                            Ok(patch_str) => {
                                match ron::from_str::<Arc<RwLock<Patch>>>(&patch_str) {
                                    Ok(patch) => {
                                        patches.insert(name, patch);
                                    }
                                    Err(e) => {
                                        warn!("Failed to deserialize patch from {:?}: {}", path, e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to read patch file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        // Load state file
        let state_file_path = project.state_file(APP_ID);
        if !state_file_path.exists() {
            warn!(
                "No persisted state file found at {:?}, performing an empty save.",
                state_file_path
            );
            self.persist_sheet_blocking()?;
            return Err(());
        }

        let state_str = fs::read_to_string(&state_file_path).map_err(|e| {
            warn!(
                "Failed to read persisted state from file {:?}: {}",
                state_file_path, e
            );
        })?;

        let mut persisted: PersistedState = ron::from_str(&state_str).map_err(|e| {
            warn!(
                "Failed to deserialize persisted state from file {:?}: {}",
                state_file_path, e
            );
        })?;

        // Fix ordering
        persisted
            .ordering
            .retain(|patch_name| patches.contains_key(patch_name));

        // Add patch IDs for patches that exist but aren't in ordering
        let mut patches_in_ordering: std::collections::HashSet<_> =
            persisted.ordering.iter().cloned().collect();

        for name in patches.keys() {
            if !patches_in_ordering.contains(name) {
                persisted.ordering.push(name.clone());
                patches_in_ordering.insert(name.clone());
            }
        }

        self.state.sheet_from_persisted(persisted, patches);

        Ok(())
    }
}

// LYN: Main App UI Implementation

const DEFAULT_SNARL_STYLE: SnarlStyle = SnarlStyle {
    pin_placement: Some(PinPlacement::Edge),
    ..SnarlStyle::new()
};

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.performance
            .update_frame_history(ctx.input(|i| i.time), frame.info().cpu_usage);

        if self.state.get_project().is_none() {
            self.draw_project_selector_ui(ctx);
        } else {
            if !self.state.states_loaded() && self.restore_sheet_blocking().is_ok() {
                self.processor_cmd_tx
                    .send(processor::Command::RebuildGraph)
                    .expect("Processor commanding channel unexpectedly closed");
            }
            self.draw_studio_ui(ctx);
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
                ui.toggle_value(&mut self.state.dsp_value_mut(), "DSP");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut port_guard = self.state.sheet_port_mut();
                    let (port, public) = &mut *port_guard;

                    ui.checkbox(public, "公开");
                    if ui
                        .add(
                            egui::DragValue::new(port)
                                .range(0..=u16::MAX)
                                .speed(1)
                                .prefix("端口 "),
                        )
                        .changed()
                    {
                        self.state.port_listener_stop();
                    }

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
            .frame(egui::Frame::NONE.fill(ctx.style().visuals.extreme_bg_color))
            .show(ctx, |ui| {
                let Some((_, patch)) = self.state.selected_patch() else {
                    if ui.response().hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
                    }
                    return;
                };

                let Patch { snarl, .. } = &mut *patch.write();
                let mut patch_viewer = PatchViewer::new(self.state.clone());
                SnarlWidget::new()
                    .id(WidgetId::MainAppCentralSnarlCanvas.into())
                    .style(DEFAULT_SNARL_STYLE)
                    .show(snarl, &mut patch_viewer, ui);
                let PatchViewerOutput { rebuild } = patch_viewer.output;
                if rebuild {
                    self.processor_cmd_tx
                        .send(processor::Command::RebuildGraph)
                        .expect("Processor commanding channel unexpectedly closed");
                }
            });
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

    fn explorer(&mut self, ui: &mut egui::Ui) {
        ui.add_sized(
            [ui.available_width(), 0.],
            egui::TextEdit::singleline(&mut self.new_patch_name),
        );

        let disable_add_button =
            self.new_patch_name.is_empty() || self.state.sheet_patch_has_name(&self.new_patch_name);
        ui.add_enabled_ui(!disable_add_button, |ui| {
            if ui
                .add_sized([ui.available_width(), 30.], egui::Button::new("添加音图"))
                .clicked()
            {
                self.state.sheet_add_patch(self.new_patch_name.take());
            };
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_be_removed = Vec::new();
            dnd(ui, WidgetId::MainAppExplorerPatchesOrderingDnd).show_vec(
                &mut self.state.sheet_patches_ordering_mut(),
                |ui, patch_name, handle, _state| {
                    let Some(arc) = self.state.sheet_get_patch(patch_name) else {
                        return;
                    };
                    let guard = arc.read();
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing = emath::vec2(4., 0.);
                        let pat_color = guard.color;

                        handle.ui(ui, |ui| {
                            ui.add_sized(
                                [46., ui.available_height()],
                                egui::Button::new(
                                    egui::RichText::new(&guard.icon)
                                        .heading()
                                        .color(pat_color.lerp_to_gamma(text_color(pat_color), 0.6)),
                                )
                                .fill(pat_color),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button(egui::RichText::new(" ").heading()).clicked() {
                                to_be_removed.push(patch_name.clone());
                            }

                            let pat_button = ui.add_sized(
                                ui.available_size(),
                                egui::Button::new(&*patch_name)
                                    .right_text("")
                                    .selected(
                                        self.state
                                            .selected_patch_name()
                                            .as_ref()
                                            .is_some_and(|name| name == patch_name),
                                    )
                                    .frame_when_inactive(true),
                            );
                            if pat_button.clicked() {
                                self.state.select_patch(Some(patch_name.clone()));
                            };
                        });
                    });
                },
            );
            for pat_name in to_be_removed {
                self.state.sheet_del_patch(&pat_name);
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

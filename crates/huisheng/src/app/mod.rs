use std::{
    sync::{Arc, mpsc},
    thread,
};

use egui_dnd::dnd;
use egui_snarl::ui::{PinPlacement, SnarlStyle, SnarlWidget};
use lyn_util::{egui::text_color, types::WithId};

use crate::{
    app::patch_viewer::PatchViewerOutput,
    model::{patch::Patch, state::CentralState},
    routines::{RoutineId, guardian, processor},
};

use self::{helpers::WidgetId, patch_viewer::PatchViewer, widgets::performance::Performance};

pub mod helpers;
pub mod patch_viewer;
pub mod widgets;

// LYN: Main App State Holder

#[derive(Debug)]
pub struct MainApp {
    // widget states
    performance: Performance,

    processor_cmd_tx: mpsc::Sender<processor::Command>,
    state: Arc<CentralState>,
}

impl MainApp {
    pub fn prepare() -> Self {
        let (processor_cmd_tx, processor_cmd_rx) = mpsc::channel();
        let state = Arc::new(CentralState::init());

        let routines = vec![(
            RoutineId::Processor,
            thread::spawn({
                let state = state.clone();
                move || processor::main(state, processor_cmd_rx)
            }),
        )];
        thread::spawn({
            let state = state.clone();
            move || guardian::main(state, routines)
        });

        Self {
            performance: Default::default(),
            processor_cmd_tx,
            state,
        }
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

        self.draw_ui(ctx);

        // ctx.request_repaint(); // Uncomment this for continuous repainting (fix some UI update issues)
    }
}

impl MainApp {
    fn draw_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top(WidgetId::MainAppTopToolBar).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("left");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("right");
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
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let Some(WithId { item: patch, .. }) = self.state.selected_patch() else {
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
                    self.processor_cmd_tx.send(processor::Command::RebuildGraph);
                }
            });
    }
}

impl MainApp {
    fn explorer(&mut self, ui: &mut egui::Ui) {
        if ui
            .add_sized([ui.available_width(), 30.], egui::Button::new("添加音图"))
            .clicked()
        {
            self.state.add_patch();
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_be_removed = Vec::new();
            dnd(ui, WidgetId::MainAppExplorerPatchesOrderingDnd).show_vec(
                &mut self.state.patches_ordering_mut(),
                |ui, patch_id, handle, _state| {
                    let Some(arc) = self.state.get_patch(patch_id) else {
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
                                to_be_removed.push(patch_id.clone());
                            }

                            let pat_button = ui.add_sized(
                                ui.available_size(),
                                egui::Button::new(&guard.name)
                                    .right_text("")
                                    .selected(
                                        self.state
                                            .selected_patch_id()
                                            .as_ref()
                                            .is_some_and(|id| id == patch_id),
                                    )
                                    .frame_when_inactive(true),
                            );
                            if pat_button.clicked() {
                                self.state.select_patch(Some(patch_id.clone()));
                            };
                        });
                    });
                },
            );
            for pat_id in to_be_removed {
                self.state.del_patch(&pat_id);
            }
        });
    }
}

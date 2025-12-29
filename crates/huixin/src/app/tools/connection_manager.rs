use std::sync::Arc;

use egui_dnd::dnd;
use lyn_util::egui::LynId;
use parking_lot::RwLock;

use crate::{
    app::{
        helpers::WidgetId,
        tools::{ToolWindow, ToolWindowId},
    },
    model::{comm::CommTarget, state::CentralState},
};

#[derive(Debug)]
pub struct ConnectionManager {
    open: bool,
    state: Arc<CentralState>,
}

impl ConnectionManager {
    pub fn new(state: Arc<CentralState>) -> Self {
        Self { open: false, state }
    }
}

impl ToolWindow for ConnectionManager {
    fn tool_id(&self) -> ToolWindowId {
        ToolWindowId::ConnectionManager
    }
    fn icon(&self) -> String {
        "󱘖 ".to_string()
    }

    fn window_open(&self) -> bool {
        self.open
    }

    fn window_open_mut(&mut self) -> &mut bool {
        &mut self.open
    }

    fn toggle_open(&mut self, open: Option<bool>) {
        if let Some(open) = open {
            self.open = open;
        } else {
            self.open = !self.open;
        }
    }
    fn draw(&mut self, ctx: &egui::Context) {
        let mut open = self.open;
        egui::Window::new("连接管理")
            .id(WidgetId::ConnectionManager.into())
            .frame(egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::ZERO))
            .collapsible(true)
            .resizable([false, true])
            .open(&mut open)
            .min_size(emath::vec2(300., 150.))
            .default_size(emath::vec2(400., 300.))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Frame::new()
                        .inner_margin(ui.style().spacing.item_spacing)
                        .show(ui, |ui| {
                            let mut to_be_removed = Vec::new();

                            let mut ordering_id_to_be_removed = Vec::new();
                            dnd(ui, WidgetId::ConnectionManagerTargetsOrderingDnd).show_vec(
                                &mut self.state.sheet_targets_ordering_mut(),
                                |ui, id, handle, state| {
                                    let Some(arc) = self.state.sheet_get_comm_target(id) else {
                                        ordering_id_to_be_removed.push(id.clone());
                                        return;
                                    };
                                    let Some(mut guard) = arc.try_write() else {
                                        ui.horizontal(|ui| {
                                            // TODO: better locking indication
                                            ui.label("editing");
                                        });
                                        return;
                                    };
                                    ui.horizontal(|ui| {
                                        handle.ui(ui, |ui| {
                                            ui.label(egui::RichText::new("󰇝").heading());
                                        });

                                        let target_id = id.clone();
                                        ui.label(if self.state.comm_stream_exists(&target_id) {
                                            egui::RichText::new(" ")
                                        } else {
                                            egui::RichText::new(" ").color(ecolor::Color32::RED)
                                        });
                                        ui.add_sized(
                                            [80., ui.available_height()],
                                            egui::TextEdit::singleline(&mut guard.name),
                                        );
                                        let addr_resp = ui.add_sized(
                                            [140., ui.available_height()],
                                            egui::TextEdit::singleline(&mut guard.addr),
                                        );
                                        let format_changed = egui::ComboBox::new(&target_id, "")
                                            .selected_text(guard.format.to_string())
                                            .show_ui(ui, |ui| {
                                                let mut changed = false;
                                                for format in lyn_util::comm::Format::variants() {
                                                    changed |= ui
                                                        .selectable_value(
                                                            &mut guard.format,
                                                            *format,
                                                            format.to_string(),
                                                        )
                                                        .clicked();
                                                }
                                                changed
                                            })
                                            .inner;
                                        if addr_resp.changed() || format_changed.is_some_and(|v| v)
                                        {
                                            self.state.comm_drop_stream(&target_id);
                                        }

                                        if ui.button(" ").clicked() {
                                            to_be_removed.push(target_id.clone());
                                        }
                                    });
                                },
                            );
                            for id in ordering_id_to_be_removed {
                                self.state.sheet_targets_ordering_mut().retain(|x| x != &id);
                            }
                            for id in to_be_removed {
                                self.state.sheet_del_comm_target(&id);
                                self.state.comm_drop_stream(&id);
                            }
                            if ui.button("新增通讯目标").clicked() {
                                self.state.sheet_add_comm_target();
                            };
                        });
                });
                ui.allocate_space(emath::vec2(350., ui.available_height()));
            });
        self.open = open;
    }
}

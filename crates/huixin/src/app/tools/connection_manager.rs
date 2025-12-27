use std::sync::Arc;

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
                            for entry in self.state.sheet_comm_targets_iter() {
                                let Some(mut guard) = entry.try_write() else {
                                    ui.horizontal(|ui| {
                                        ui.label("editing");
                                    });
                                    continue;
                                };
                                ui.horizontal(|ui| {
                                    let id = entry.key().clone();
                                    ui.label(if self.state.comm_stream_exists(&id) {
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
                                    let format_changed = egui::ComboBox::new(entry.key(), "")
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
                                    if addr_resp.changed() || format_changed.is_some_and(|v| v) {
                                        self.state.comm_drop_stream(&id);
                                    }

                                    if ui.button(" ").clicked() {
                                        to_be_removed.push(entry.key().clone());
                                    }
                                });
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

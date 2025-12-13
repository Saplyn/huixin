use std::sync::Arc;

use lyn_util::egui::LynId;
use parking_lot::RwLock;

use crate::{
    app::{CommonState, helpers::WidgetId, tools::ToolWindow},
    model::CommTarget,
    routines::instructor::Instructor,
};

#[derive(Debug)]
pub struct ConnectionManager {
    // ui states
    open: bool,

    // logic states
    common: Arc<CommonState>,
    instructor: Arc<Instructor>,
}

impl ConnectionManager {
    pub fn new(common: Arc<CommonState>, instructor: Arc<Instructor>) -> Self {
        Self {
            open: false,
            common,
            instructor,
        }
    }
}

impl ToolWindow for ConnectionManager {
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
            .collapsible(false)
            .open(&mut open)
            .min_size(emath::vec2(300., 150.))
            .default_size(emath::vec2(400., 300.))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Frame::new()
                        .inner_margin(ui.style().spacing.item_spacing)
                        .show(ui, |ui| {
                            let targets = self.instructor.targets();
                            let mut to_be_removed = Vec::new();
                            for entry in targets.iter() {
                                let mut guard = entry.write();
                                ui.horizontal(|ui| {
                                    ui.label(if guard.stream.is_some() {
                                        " "
                                    } else {
                                        " "
                                    });
                                    ui.add_sized(
                                        emath::vec2(100., ui.available_height()),
                                        egui::TextEdit::singleline(&mut guard.name),
                                    );
                                    if ui
                                        .add_sized(
                                            emath::vec2(200., ui.available_height()),
                                            egui::TextEdit::singleline(&mut guard.addr),
                                        )
                                        .changed()
                                    {
                                        guard.stream = None;
                                    }
                                    if ui.button(" ").clicked() {
                                        to_be_removed.push(*entry.key());
                                    }
                                });
                            }
                            for id in to_be_removed {
                                targets.remove(&id);
                            }
                            if ui.button("新增通讯目标").clicked() {
                                targets.insert(
                                    LynId::obtain_id(),
                                    Arc::new(RwLock::new(CommTarget::default())),
                                );
                            };
                        });
                });
                ui.allocate_space(emath::vec2(350., ui.available_height()));
            });
        self.open = open;
    }
}

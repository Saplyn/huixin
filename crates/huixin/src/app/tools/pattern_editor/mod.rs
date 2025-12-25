use std::{ops::DerefMut, sync::Arc};

use crate::{
    app::{
        helpers::WidgetId,
        tools::{ToolWindow, ToolWindowId, pattern_editor::midi_editor::MidiEditor},
    },
    model::{pattern::SheetPattern, state::CentralState},
};

mod midi_editor;

#[derive(Debug)]
pub struct PatternEditor {
    open: bool,
    state: Arc<CentralState>,
}

impl PatternEditor {
    pub fn new(state: Arc<CentralState>) -> Self {
        Self { open: false, state }
    }
}

impl ToolWindow for PatternEditor {
    fn tool_id(&self) -> ToolWindowId {
        ToolWindowId::PatternEditor
    }
    fn icon(&self) -> String {
        "󰎅 ".to_string()
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
        egui::Window::new("片段编辑")
            .id(WidgetId::PatternEditor.into())
            .frame(egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::ZERO))
            .collapsible(true)
            .open(&mut open)
            .min_size(emath::vec2(300., 150.))
            .default_size(emath::vec2(400., 300.))
            .show(ctx, |ui| {
                let Some(pat) = self.state.selected_pattern() else {
                    ui.disable();
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.label(egui::RichText::new("请选择一个片段以编辑").heading());
                        },
                    );
                    return;
                };

                match pat.write().deref_mut() {
                    SheetPattern::Midi(pat) => {
                        MidiEditor::new(pat, self.state.clone()).show_inside(ui)
                    }
                };
            });
        self.open = open;
    }
}

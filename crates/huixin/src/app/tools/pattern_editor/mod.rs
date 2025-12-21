use std::{ops::DerefMut, sync::Arc};

use crate::{
    app::{
        CommonState,
        helpers::WidgetId,
        tools::{
            ToolWindow, ToolWindowId,
            pattern_editor::midi_editor::{MidiEditor, MidiEditorState},
        },
    },
    model::pattern::SheetPattern,
    routines::{instructor::Instructor, metronome::Metronome, sheet_reader::SheetReader},
};

mod midi_editor;

#[derive(Debug)]
pub struct PatternEditor {
    // ui states
    open: bool,
    midi_editor_state: MidiEditorState,

    // logic states
    common: Arc<CommonState>,
    metronome: Arc<Metronome>,
    sheet_reader: Arc<SheetReader>,
    instructor: Arc<Instructor>,
}

impl PatternEditor {
    pub fn new(
        common: Arc<CommonState>,
        metronome: Arc<Metronome>,
        sheet_reader: Arc<SheetReader>,
        instructor: Arc<Instructor>,
    ) -> Self {
        Self {
            open: false,
            midi_editor_state: MidiEditorState::default(),
            common,
            metronome,
            sheet_reader,
            instructor,
        }
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
                let Some(pat) = self.common.selected_pattern(self.sheet_reader.clone()) else {
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
                        MidiEditor::new(&mut self.midi_editor_state, pat, self.instructor.targets())
                            .show_inside(ui)
                    }
                };
            });
        self.open = open;
    }
}

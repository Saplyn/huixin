use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use self::midi_editor::{constants::KEY_ROW_HEIGHT, midi_keyboard::MidiKeyboard};

use crate::{
    app::{
        MainState,
        helpers::WidgetId,
        tools::{
            ToolWindow,
            pattern_editor::midi_editor::{MidiEditor, constants::NUMBER_OF_KEYS},
        },
    },
    routines::{metronome::Metronome, sheet_reader::SheetReader},
    sheet::pattern::SheetPatternInner,
};

mod midi_editor;

// LYN: Pattern Editor Main Interface

#[derive(Debug)]
pub struct PatternEditor {
    // ui states
    open: bool,

    // logic states
    main_state: Arc<MainState>,
    metronome: Arc<Metronome>,
    sheet_reader: Arc<SheetReader>,
}

impl PatternEditor {
    pub fn new(
        main_state: Arc<MainState>,
        metronome: Arc<Metronome>,
        sheet_reader: Arc<SheetReader>,
    ) -> Self {
        Self {
            open: true,
            main_state,
            metronome,
            sheet_reader,
        }
    }
}

impl ToolWindow for PatternEditor {
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
        egui::Window::new("Pattern Editor")
            .id(WidgetId::PatternEditor.into())
            .frame(egui::Frame::window(&ctx.style()).inner_margin(0))
            .collapsible(false)
            .open(&mut open)
            .min_size(emath::vec2(300., 150.))
            .default_size(emath::vec2(400., 300.))
            .show(ctx, |ui| {
                let pat_guard = self.main_state.selected_pattern_mut();
                let Some(pat) = pat_guard.as_ref() else {
                    ui.disable();
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.label(egui::RichText::new("请选择一个片段以编辑").heading());
                        },
                    );
                    return;
                };

                let pat_guard = pat.upgrade();
                let Some(mut pat) = pat_guard.as_deref().map(|pat| pat.write()) else {
                    return;
                };
                match pat.deref_mut().inner {
                    SheetPatternInner::Midi(ref mut pat) => {
                        let output = MidiEditor::new(pat).show_inside(ui);
                    }
                };
            });
        self.open = open;
    }
}

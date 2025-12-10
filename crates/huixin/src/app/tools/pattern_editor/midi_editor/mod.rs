use std::cmp::min;

use self::{
    constants::KEY_ROW_HEIGHT, midi_keyboard::MidiKeyboard, midi_note::MidiNoteWidget,
    midi_rows::MidiRows,
};
use crate::{
    app::helpers::WidgetId,
    routines::metronome::TICK_PER_BEAT,
    sheet::pattern::midi::{MidiNote, MidiPattern},
};

pub mod constants;
pub mod midi_keyboard;
pub mod midi_note;
pub mod midi_rows;

const MIN_SIZE_PER_BEAT: f32 = 40.;
const MAX_SIZE_PER_BEAT: f32 = 400.;

#[derive(Debug)]
pub struct MidiEditorState {
    pub size_per_beat: f32,
}

impl Default for MidiEditorState {
    fn default() -> Self {
        Self {
            size_per_beat: MIN_SIZE_PER_BEAT,
        }
    }
}

#[derive(Debug)]
pub struct MidiEditor<'pat, 'state> {
    state: &'state mut MidiEditorState,
    midi_pattern: &'pat mut MidiPattern,
}

#[derive(Debug)]
pub struct MidiEditorOutput {}

impl<'pat, 'state> MidiEditor<'pat, 'state> {
    pub fn new(state: &'state mut MidiEditorState, midi_pattern: &'pat mut MidiPattern) -> Self {
        Self {
            state,
            midi_pattern,
        }
    }
}

impl<'pat, 'state> MidiEditor<'pat, 'state> {
    pub fn show_inside(mut self, ui: &mut egui::Ui) -> MidiEditorOutput {
        egui::TopBottomPanel::top(WidgetId::PatternEditorMidiUtilBar)
            .frame(egui::Frame::NONE.inner_margin(4.))
            .show_inside(ui, |ui| {
                self.util_bar(ui);
            });

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = emath::vec2(0., 0.);
                let midi_keyboard_resp = ui.add(MidiKeyboard);

                egui::ScrollArea::horizontal().show(ui, |ui| {
                    MidiRows::new(self.state.size_per_beat, self.midi_pattern).show(ui);

                    let notes = self.midi_pattern.notes_iter_owned().collect::<Vec<_>>();
                    for note in notes {
                        MidiNoteWidget::new(
                            note.id(),
                            self.midi_pattern,
                            note,
                            self.state.size_per_beat,
                            TICK_PER_BEAT / 4,
                        )
                        .show(ui);
                    }
                });
            })
        });

        MidiEditorOutput {}
    }
}

impl<'pat, 'state> MidiEditor<'pat, 'state> {
    fn util_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("util bar");
            // TODO: calc & impl actual max location
            // let max_loc = 128;
            // ui.add(egui::DragValue::new(&mut self.length).range(0..=max_loc));
            // ui.spacing_mut().slider_width = ui.available_width();
            // ui.add(egui::Slider::new(&mut self.offset, 0..=max_loc).show_value(false));
        });
    }

    fn right_detail_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("right");
    }
}

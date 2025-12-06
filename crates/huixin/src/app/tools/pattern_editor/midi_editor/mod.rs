use std::cmp::min;

use self::{
    constants::KEY_ROW_HEIGHT, midi_keyboard::MidiKeyboard, midi_note::MidiNote,
    midi_rows::MidiRows,
};
use crate::{
    app::helpers::WidgetId, routines::metronome::TICK_PER_BEAT, sheet::pattern::midi::MidiPattern,
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

        let output = egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = emath::vec2(0., 0.);
                let midi_keyboard_resp = ui.add(MidiKeyboard);
                let midi_rows_resp = egui::ScrollArea::horizontal()
                    .show(ui, |ui| {
                        let midi_rows_resp =
                            ui.add(MidiRows::new(self.state.size_per_beat, self.midi_pattern));

                        for note in self.midi_pattern.notes_mut().iter_mut() {
                            let anchor = ui.min_rect().left_top();
                            let min = egui::Pos2 {
                                x: anchor.x
                                    + note.start as f32 / TICK_PER_BEAT as f32
                                        * self.state.size_per_beat,
                                y: anchor.y + (127 - note.midicode) as f32 * KEY_ROW_HEIGHT,
                            };
                            let max = egui::Pos2 {
                                x: anchor.x
                                    + (note.start + note.length) as f32 / TICK_PER_BEAT as f32
                                        * self.state.size_per_beat,
                                y: anchor.y + (128 - note.midicode) as f32 * KEY_ROW_HEIGHT,
                            };
                            let note_resp = ui.put(egui::Rect { min, max }, MidiNote);
                        }

                        midi_rows_resp
                    })
                    .inner;

                (midi_keyboard_resp, midi_rows_resp)
            })
            .inner
        });

        let (midi_keyboard_resp, midi_rows_resp) = output.inner;
        let scroll_offset = output.state.offset;
        if let Some(row) = midi_rows_resp.interact_pointer_pos()
            && (midi_rows_resp.clicked() || midi_rows_resp.dragged())
        {
            let key_id = 127
                - min(
                    ((row.y + scroll_offset.y - output.inner_rect.top()) / KEY_ROW_HEIGHT).floor()
                        as u32,
                    127,
                );
            println!("Clicked key id: {}", key_id);
        }
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

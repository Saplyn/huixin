use self::{midi_keyboard::MidiKeyboard, midi_note::MidiNoteWidget, midi_rows::MidiRows};
use crate::{
    app::helpers::WidgetId, routines::metronome::TICK_PER_BEAT, sheet::pattern::midi::MidiPattern,
};

pub mod constants;
pub mod midi_keyboard;
pub mod midi_note;
pub mod midi_rows;

const MIN_SIZE_PER_BEAT: f32 = 40.;
const MAX_SIZE_PER_BEAT: f32 = 400.;

// LYN: Editor Externally Held State

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

// LYN: Midi Editor State

#[derive(Debug)]
pub struct MidiEditor<'pat, 'state> {
    state: &'state mut MidiEditorState,
    midi_pattern: &'pat mut MidiPattern,
}

impl<'pat, 'state> MidiEditor<'pat, 'state> {
    pub fn new(state: &'state mut MidiEditorState, midi_pattern: &'pat mut MidiPattern) -> Self {
        Self {
            state,
            midi_pattern,
        }
    }
}

impl<'pat, 'state> MidiEditor<'pat, 'state> {
    pub fn show_inside(mut self, ui: &mut egui::Ui) {
        egui::SidePanel::right(WidgetId::PatternEditorMidiDetailPanel)
            .resizable(false)
            .min_width(150.)
            .default_width(150.)
            .show_inside(ui, |ui| {
                self.detail_panel(ui);
            });

        egui::TopBottomPanel::top(WidgetId::PatternEditorMidiUtilBar)
            .frame(egui::Frame::NONE.inner_margin(4.))
            .show_inside(ui, |ui| {
                self.util_bar(ui);
            });

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = emath::vec2(0., 0.);
                MidiKeyboard.show(ui);

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
    }
}

impl<'pat, 'state> MidiEditor<'pat, 'state> {
    fn util_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("util bar");
        });
    }

    fn detail_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("图标：");
            ui.add(egui::TextEdit::singleline(&mut self.midi_pattern.icon).char_limit(2));
        });

        ui.horizontal(|ui| {
            ui.label("名称：");
            ui.add(egui::TextEdit::singleline(&mut self.midi_pattern.name));
        });

        ui.horizontal(|ui| {
            ui.label("长度：");
            let min_beats = self.midi_pattern.min_beats();
            ui.add(
                egui::DragValue::new(&mut self.midi_pattern.beats)
                    .range(min_beats..=(u64::MAX >> 2)),
            );
        });
    }
}

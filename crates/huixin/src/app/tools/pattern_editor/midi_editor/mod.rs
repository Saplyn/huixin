use std::sync::Arc;

use dashmap::DashMap;
use lyn_util::egui::LynId;
use parking_lot::RwLock;

use self::{midi_keyboard::MidiKeyboard, midi_note::MidiNoteWidget, midi_rows::MidiRows};
use crate::{
    app::helpers::WidgetId,
    model::{CommTarget, pattern::midi::MidiPattern},
    routines::metronome::TICK_PER_BEAT,
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
pub struct MidiEditor<'pat, 'state, 'targets> {
    state: &'state mut MidiEditorState,
    midi_pattern: &'pat mut MidiPattern,
    targets: &'targets DashMap<String, Arc<RwLock<CommTarget>>>,
}

impl<'pat, 'state, 'targets> MidiEditor<'pat, 'state, 'targets> {
    pub fn new(
        state: &'state mut MidiEditorState,
        midi_pattern: &'pat mut MidiPattern,
        targets: &'targets DashMap<String, Arc<RwLock<CommTarget>>>,
    ) -> Self {
        Self {
            state,
            midi_pattern,
            targets,
        }
    }
}

impl<'pat, 'state, 'targets> MidiEditor<'pat, 'state, 'targets> {
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

impl<'pat, 'state, 'targets> MidiEditor<'pat, 'state, 'targets> {
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

        ui.horizontal(|ui| {
            ui.label("标识：");
            ui.add(egui::TextEdit::singleline(&mut self.midi_pattern.tag));
        });

        ui.horizontal(|ui| {
            ui.label("目标：");

            let target_name = self
                .midi_pattern
                .target_id
                .as_ref()
                .and_then(|id| self.targets.get(id))
                .map(|target| target.read().name.clone());
            if target_name.is_none() {
                self.midi_pattern.target_id = None;
            }
            egui::ComboBox::from_label("target")
                .selected_text(target_name.unwrap_or("未选择".to_string()))
                .show_ui(ui, |ui| {
                    let target_id_mut = &mut self.midi_pattern.target_id;
                    for entry in self.targets.iter() {
                        ui.selectable_value(
                            target_id_mut,
                            Some(entry.key().clone()),
                            &entry.value().read().name,
                        );
                    }
                })
        });
    }
}

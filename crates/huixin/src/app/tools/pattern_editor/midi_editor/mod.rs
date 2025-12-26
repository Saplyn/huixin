use std::sync::Arc;

use egui_winit::clipboard::Clipboard;

use self::{midi_keyboard::MidiKeyboard, midi_note::MidiNoteWidget, midi_rows::MidiRows};
use crate::{
    app::helpers::WidgetId,
    model::{
        pattern::{SheetPatternTrait, midi::MidiPattern},
        state::CentralState,
    },
    routines::metronome::TICK_PER_BEAT,
};

pub mod constants;
pub mod midi_keyboard;
pub mod midi_note;
pub mod midi_rows;

// LYN: Midi Editor State

#[derive(Debug)]
pub struct MidiEditor<'pat> {
    midi_pattern: &'pat mut MidiPattern,
    state: Arc<CentralState>,
}

impl<'pat> MidiEditor<'pat> {
    pub fn new(midi_pattern: &'pat mut MidiPattern, state: Arc<CentralState>) -> Self {
        Self {
            midi_pattern,
            state,
        }
    }
}

impl<'pat> MidiEditor<'pat> {
    pub fn show_inside(mut self, ui: &mut egui::Ui) {
        egui::SidePanel::right(WidgetId::PatternEditorMidiDetailPanel)
            .resizable(false)
            .show_inside(ui, |ui| {
                self.detail_panel(ui);
            });

        if !self.midi_pattern.usable() {
            egui::TopBottomPanel::top(WidgetId::PatternEditorMidiNotificationBar)
                .frame(egui::Frame::NONE.inner_margin(4.))
                .show_inside(ui, |ui| {
                    ui.colored_label(
                        ui.style().visuals.error_fg_color,
                        "请确保已通讯选择目标且标识不为空。",
                    );
                });
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = emath::vec2(0., 0.);
                MidiKeyboard.show(ui);

                let size_per_beat = *self.state.ui.pattern_editor_size_per_beat.read();
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    MidiRows::new(size_per_beat, self.midi_pattern).show(ui);

                    let notes = self.midi_pattern.notes_iter_owned().collect::<Vec<_>>();
                    for note in notes {
                        MidiNoteWidget::new(
                            self.midi_pattern,
                            note,
                            size_per_beat,
                            TICK_PER_BEAT / 4,
                        )
                        .show(ui);
                    }
                });
            })
        });
    }
}

impl<'pat> MidiEditor<'pat> {
    fn detail_panel(&mut self, ui: &mut egui::Ui) {
        let width = 95.;
        ui.horizontal(|ui| {
            ui.label("图标：");
            ui.add_sized(
                [width, ui.available_height()],
                egui::TextEdit::singleline(&mut self.midi_pattern.icon).char_limit(2),
            );
        });

        ui.horizontal(|ui| {
            ui.label("名称：");
            ui.add_sized(
                [width, ui.available_height()],
                egui::TextEdit::singleline(&mut self.midi_pattern.name),
            );
        });

        ui.horizontal(|ui| {
            ui.label("颜色：");
            egui::color_picker::color_edit_button_srgba(
                ui,
                &mut self.midi_pattern.color,
                egui::color_picker::Alpha::Opaque,
            );
            if ui.button("󰆏 ").clicked() {
                let (r, g, b, _) = self.midi_pattern.color.to_tuple();
                let hex = format!("#{r:02X}{g:02X}{b:02X}");
                Clipboard::new(None).set_text(hex);
            }
            if ui.button("󰆒 ").clicked()
                && let Some(text) = Clipboard::new(None).get()
            {
                let s = text.trim().trim_start_matches('#');
                if s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit()) {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&s[0..2], 16),
                        u8::from_str_radix(&s[2..4], 16),
                        u8::from_str_radix(&s[4..6], 16),
                    ) {
                        self.midi_pattern.color = ecolor::Color32::from_rgb(r, g, b);
                    }
                } else {
                    let parts: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
                    if parts.len() == 3
                        && let (Ok(r), Ok(g), Ok(b)) = (
                            parts[0].parse::<u8>(),
                            parts[1].parse::<u8>(),
                            parts[2].parse::<u8>(),
                        )
                    {
                        self.midi_pattern.color = ecolor::Color32::from_rgb(r, g, b);
                    }
                }
            };
        });

        ui.horizontal(|ui| {
            ui.label("长度：");
            let min_beats = self.midi_pattern.min_beats();
            ui.add_sized(
                [width, ui.available_height()],
                egui::DragValue::new(&mut self.midi_pattern.beats)
                    .range(min_beats..=(u64::MAX >> 2)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("标识：");
            ui.add_sized(
                [width, ui.available_height()],
                egui::TextEdit::singleline(&mut self.midi_pattern.tag),
            );
        });

        ui.horizontal(|ui| {
            ui.label("目标：");

            let target_name = self
                .midi_pattern
                .target_id
                .as_ref()
                .and_then(|id| self.state.sheet_get_comm_target(id))
                .map(|target| target.read().name.clone());
            if target_name.is_none() {
                self.midi_pattern.target_id = None;
            }
            let target_name = {
                let name = target_name.unwrap_or("未选择".to_string());
                let chars = name.chars();
                if chars.clone().count() <= 4 {
                    name
                } else {
                    name.chars().take(3).chain("…".chars()).collect()
                }
            };
            egui::ComboBox::new(WidgetId::PatternEditorMidiComboBoxCommTarget, "")
                .selected_text(target_name)
                .width(width)
                .show_ui(ui, |ui| {
                    let target_id_mut = &mut self.midi_pattern.target_id;
                    for entry in self.state.sheet_comm_targets_iter() {
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

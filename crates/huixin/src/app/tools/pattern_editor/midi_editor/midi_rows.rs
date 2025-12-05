use crate::sheet::pattern::MidiPattern;

use super::constants::{KEY_ROW_HEIGHT, NUMBER_OF_KEYS};

#[derive(Debug)]
pub struct MidiRows<'pat> {
    midi_pattern: &'pat mut MidiPattern,
}

impl<'pat> MidiRows<'pat> {
    pub fn new(midi_pattern: &'pat mut MidiPattern) -> Self {
        Self { midi_pattern }
    }
}

impl<'pat> egui::Widget for MidiRows<'pat> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size =
            emath::vec2(ui.available_width(), NUMBER_OF_KEYS as f32 * KEY_ROW_HEIGHT);
        let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::all());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            for key in 0..=NUMBER_OF_KEYS {
                let y = rect.top() + key as f32 * KEY_ROW_HEIGHT;
                let line_start = emath::pos2(rect.left(), y);
                let line_end = emath::pos2(rect.right(), y);
                painter.line_segment(
                    [line_start, line_end],
                    egui::Stroke::new(1.0, ecolor::Color32::DARK_GRAY),
                );
            }
        }

        resp
    }
}

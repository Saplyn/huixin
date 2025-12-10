use crate::sheet::pattern::{SheetPatternTrait, midi::MidiPattern};

use super::constants::{KEY_ROW_HEIGHT, NUMBER_OF_KEYS};

#[derive(Debug)]
pub struct MidiRows<'pat> {
    size_per_beat: f32,
    midi_pattern: &'pat mut MidiPattern,
}

impl<'pat> MidiRows<'pat> {
    pub fn new(size_per_beat: f32, midi_pattern: &'pat mut MidiPattern) -> Self {
        Self {
            size_per_beat,
            midi_pattern,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let total_width = self.midi_pattern.beats() as f32 * self.size_per_beat;
        let desired_size = emath::vec2(total_width, NUMBER_OF_KEYS as f32 * KEY_ROW_HEIGHT);
        let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

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

        // DEBUG: Map pointer position to key id on click or drag
        if let Some(pos) = resp.interact_pointer_pos()
            && (resp.clicked() || resp.dragged())
        {
            // Translate global pointer pos to local row space
            let local_y = pos.y - rect.top();
            let key_id = 127 - ((local_y / KEY_ROW_HEIGHT).floor() as u32).min(127);
            println!("Clicked key id: {}", key_id);
        }
    }
}

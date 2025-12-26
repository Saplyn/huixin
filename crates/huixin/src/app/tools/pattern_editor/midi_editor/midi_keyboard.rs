use super::constants::{
    BLACK_KEY_WIDTH_SCALE, KEY_ROW_HEIGHT, KEY_ROW_WIDTH, NUMBER_OF_BLACK_KEYS, NUMBER_OF_KEYS,
    NUMBER_OF_WHITE_KEYS,
};

#[derive(Debug)]
#[must_use]
pub struct MidiKeyboard;

impl MidiKeyboard {
    pub const WHITE_KEY_COLOR: ecolor::Color32 = ecolor::Color32::WHITE;
    pub const BLACK_KEY_COLOR: ecolor::Color32 = ecolor::Color32::BLACK;

    pub fn show(self, ui: &mut egui::Ui) {
        let desired_size = emath::vec2(KEY_ROW_WIDTH, NUMBER_OF_KEYS as f32 * KEY_ROW_HEIGHT);
        let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::all());

        let visuals = ui.style().noninteractive();

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);

            // white keys
            let mut white_key_y_offset = 0.;
            for key in (0..NUMBER_OF_WHITE_KEYS).rev() {
                let index = if key == 75 - 1 { 0 } else { (key + 4) % 7 };
                let height = match index {
                    0 | 3 | 4 | 6 => 1.5,
                    1 | 2 | 5 => 2.,
                    _ => unreachable!(),
                } * KEY_ROW_HEIGHT;

                let y = rect.top() + white_key_y_offset;
                let key_rect = emath::Rect::from_min_size(
                    emath::pos2(rect.left(), y),
                    emath::vec2(rect.width(), height),
                );
                painter.rect_filled(key_rect, 0.0, Self::WHITE_KEY_COLOR);
                painter.rect_stroke(
                    key_rect,
                    0.0,
                    (1.0, visuals.fg_stroke.color),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    emath::pos2(key_rect.right_center().x - 2., key_rect.right_center().y),
                    egui::Align2::RIGHT_CENTER,
                    white_key_id_to_midi_num(key),
                    egui::FontId::default(),
                    visuals.text_color(),
                );

                white_key_y_offset += height;
            }

            // black keys
            let mut black_key_y_offset = -KEY_ROW_HEIGHT;
            for key in (0..NUMBER_OF_BLACK_KEYS).rev() {
                let index = (key + 4) % 5;
                black_key_y_offset += match index {
                    1 | 2 | 4 => 2.,
                    0 | 3 => 3.,
                    _ => unreachable!(),
                } * KEY_ROW_HEIGHT;
                let y = rect.top() + black_key_y_offset;
                let key_rect = emath::Rect::from_min_size(
                    emath::pos2(rect.left(), y),
                    emath::vec2(rect.width() * BLACK_KEY_WIDTH_SCALE, KEY_ROW_HEIGHT),
                );
                painter.rect_filled(key_rect, 0.0, Self::BLACK_KEY_COLOR);
                painter.rect_stroke(
                    key_rect,
                    0.0,
                    (1.0, visuals.fg_stroke.color),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    emath::pos2(key_rect.right_center().x - 3., key_rect.right_center().y),
                    egui::Align2::RIGHT_CENTER,
                    black_key_id_to_midi_num(key),
                    egui::FontId::default(),
                    visuals.text_color(),
                );
            }
        }
    }
}

fn white_key_id_to_midi_num(key: u32) -> u32 {
    key / 7 * 12
        + match key % 7 {
            0 => 0,  // C
            1 => 2,  // D
            2 => 4,  // E
            3 => 5,  // F
            4 => 7,  // G
            5 => 9,  // A
            6 => 11, // B
            _ => unreachable!(),
        }
}

fn black_key_id_to_midi_num(key: u32) -> u32 {
    key / 5 * 12
        + match key % 5 {
            0 => 1,  // C#
            1 => 3,  // D#
            2 => 6,  // F#
            3 => 8,  // G#
            4 => 10, // A#
            _ => unreachable!(),
        }
}

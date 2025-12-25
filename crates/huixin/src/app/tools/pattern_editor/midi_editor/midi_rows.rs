use crate::{
    model::pattern::{
        SheetPatternTrait,
        midi::{MidiNote, MidiPattern},
    },
    routines::metronome::TICK_PER_BEAT,
};

use super::constants::{KEY_ROW_HEIGHT, NUMBER_OF_KEYS};

#[derive(Debug)]
#[must_use]
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

    fn calc_midicode(&self, y: f32) -> u8 {
        127 - ((y / KEY_ROW_HEIGHT).floor() as u8).min(127)
    }

    fn calc_tick(&self, x: f32) -> u64 {
        ((x / self.size_per_beat) * TICK_PER_BEAT as f32).floor() as u64
    }
}

impl<'pat> MidiRows<'pat> {
    pub fn show(self, ui: &mut egui::Ui) {
        let total_width = self.midi_pattern.beats() as f32 * self.size_per_beat;
        let desired_size = emath::vec2(total_width, NUMBER_OF_KEYS as f32 * KEY_ROW_HEIGHT);
        let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

        let visuals = ui.style().noninteractive();

        if resp.clicked() {
            let pos = resp.interact_pointer_pos().unwrap();
            let midicode = self.calc_midicode(pos.y - rect.top());
            let start = self.calc_tick(pos.x - rect.left());
            self.midi_pattern
                .add_note(MidiNote::new(midicode, u16::MAX, start, TICK_PER_BEAT));
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);

            // background color
            for row in 0..=NUMBER_OF_KEYS {
                let y = rect.top() + row as f32 * KEY_ROW_HEIGHT;
                let key_color = if is_black_key(row as u8) {
                    visuals.bg_fill.linear_multiply(0.5)
                } else {
                    visuals.bg_fill.linear_multiply(1.5)
                };
                let rect = emath::Rect::from_min_max(
                    emath::pos2(rect.left(), y),
                    emath::pos2(rect.right(), y + KEY_ROW_HEIGHT),
                );
                painter.rect_filled(rect, 0., key_color);
            }

            // vertical lines
            for tick in 0..=self.midi_pattern.beats() * TICK_PER_BEAT {
                let x = rect.left() + (tick as f32 / TICK_PER_BEAT as f32) * self.size_per_beat;
                let line_start = emath::pos2(x, rect.top());
                let line_end = emath::pos2(x, rect.bottom());
                painter.line_segment(
                    [line_start, line_end],
                    egui::Stroke::new(
                        match tick % (TICK_PER_BEAT * 4) {
                            0 => 0.7,
                            4 | 8 | 12 => 0.4,
                            _ => 0.2,
                        },
                        visuals.fg_stroke.color.linear_multiply(0.5),
                    ),
                );
            }

            // horizontal lines
            for key in 0..=NUMBER_OF_KEYS {
                let y = rect.top() + key as f32 * KEY_ROW_HEIGHT;
                let line_start = emath::pos2(rect.left(), y);
                let line_end = emath::pos2(rect.right(), y);
                painter.line_segment(
                    [line_start, line_end],
                    egui::Stroke::new(
                        1.,
                        visuals.fg_stroke.color.linear_multiply(if key % 12 == 8 {
                            0.8
                        } else {
                            0.3
                        }),
                    ),
                );
            }
        }
    }
}

fn is_black_key(midicode: u8) -> bool {
    matches!(midicode % 12, 1 | 4 | 6 | 9 | 11)
}

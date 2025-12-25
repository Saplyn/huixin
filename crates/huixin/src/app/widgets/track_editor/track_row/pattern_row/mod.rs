use std::sync::Arc;

use crate::{
    app::widgets::track_editor::constants::TRACK_TIMELINE_HEIGHT,
    model::{pattern::SheetPatternTrait, state::CentralState, track::pattern::PatternTrack},
    routines::metronome::TICK_PER_BEAT,
};

pub mod track_pattern;

#[derive(Debug)]
#[must_use]
pub struct PatternTrackRow<'track> {
    size_per_beat: f32,
    track: &'track mut PatternTrack,
    state: Arc<CentralState>,
}

impl<'track> PatternTrackRow<'track> {
    pub fn new(
        size_per_beat: f32,
        track: &'track mut PatternTrack,
        state: Arc<CentralState>,
    ) -> Self {
        Self {
            size_per_beat,
            track,
            state,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let length_in_beats = self.state.sheet_length_in_beats();
        let width = length_in_beats as f32 * self.size_per_beat;
        let desired_size = emath::vec2(width, TRACK_TIMELINE_HEIGHT);
        let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

        if resp.clicked()
            && let Some(pat) = self.state.selected_pattern()
        {
            let pos = resp.interact_pointer_pos().unwrap();
            let start =
                ((pos.x - rect.left()) / self.size_per_beat * TICK_PER_BEAT as f32).floor() as u64;
            let end = start + pat.item.read().beats() * TICK_PER_BEAT;
            self.track.add_pattern(start..end, pat.id);
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);

            // vertical lines
            for tick in 0..=length_in_beats * TICK_PER_BEAT {
                let x = rect.left() + (tick as f32 / TICK_PER_BEAT as f32) * self.size_per_beat;
                painter.line_segment(
                    [emath::pos2(x, rect.top()), emath::pos2(x, rect.bottom())],
                    (
                        match (tick % (TICK_PER_BEAT * 4), tick % TICK_PER_BEAT) {
                            (0, _) => 0.6,
                            (_, 0) => 0.2,
                            _ => 0.1,
                        },
                        ui.style().noninteractive().fg_stroke.color,
                    ),
                );
            }

            // border
            painter.rect_stroke(
                rect,
                0.0,
                (0.4, ui.style().noninteractive().fg_stroke.color),
                egui::StrokeKind::Inside,
            );
        }
    }
}

use std::sync::Arc;

use self::pattern_row::PatternTrackRow;
use crate::{
    app::widgets::track_editor::track_row::pattern_row::track_pattern::TrackPatternWidget,
    model::{
        state::{CentralState, WithId},
        track::SheetTrack,
    },
    routines::metronome::TICK_PER_BEAT,
};

mod pattern_row;

#[derive(Debug)]
#[must_use]
pub struct TrackRow<'track> {
    track: &'track mut SheetTrack,
    state: Arc<CentralState>,
}

impl<'track> TrackRow<'track> {
    pub fn new(track: &'track mut SheetTrack, state: Arc<CentralState>) -> Self {
        Self { track, state }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let size_per_beat = *self.state.ui.track_editor_size_per_beat.read();

        match self.track {
            SheetTrack::Pattern(track) => {
                PatternTrackRow::new(size_per_beat, track, self.state.clone()).show(ui);

                let patterns = track
                    .patterns_iter()
                    .flat_map(|(range, id_vec)| id_vec.iter().map(|id| (range.clone(), id.clone())))
                    .collect::<Vec<_>>();
                for (range, (pat_ui_id, pat_id)) in patterns {
                    let Some(arc_pat) = self.state.sheet_get_pattern(&pat_id) else {
                        continue;
                    };

                    let pat = arc_pat.read();
                    TrackPatternWidget::new(
                        size_per_beat,
                        track,
                        range,
                        WithId::new((pat_ui_id, pat_id), &pat),
                        TICK_PER_BEAT / 4,
                    )
                    .show(ui);
                }
            }
        }
    }
}

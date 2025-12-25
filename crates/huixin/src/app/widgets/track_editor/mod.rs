use std::sync::Arc;

use self::{track_header::TrackHeader, track_row::TrackRow};
use crate::model::state::CentralState;

mod constants;
mod track_header;
mod track_row;

const MIN_SIZE_PER_BEAT: f32 = 40.;
const MAX_SIZE_PER_BEAT: f32 = 400.;

#[derive(Debug)]
pub struct TrackEditor {
    state: Arc<CentralState>,
}

impl TrackEditor {
    pub fn new(state: Arc<CentralState>) -> Self {
        Self { state }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = emath::vec2(0., 0.);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for track in self.state.sheet_tracks_iter() {
                        TrackHeader::new(&track.value().write()).show(ui);
                    }
                });

                ui.vertical(|ui| {
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        for track in self.state.sheet_tracks_iter() {
                            TrackRow::new(&mut track.value().write(), self.state.clone()).show(ui);
                        }
                    });
                });
            });
        });
    }
}

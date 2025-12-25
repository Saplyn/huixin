use crate::model::track::{SheetTrack, SheetTrackTrait};

use super::constants::{TRACK_HEADER_WIDTH, TRACK_TIMELINE_HEIGHT};

#[derive(Debug)]
pub struct TrackHeader<'track> {
    track: &'track SheetTrack,
}

impl<'track> TrackHeader<'track> {
    pub fn new(track: &'track SheetTrack) -> Self {
        Self { track }
    }
    pub fn show(self, ui: &mut egui::Ui) {
        let desired_size = emath::vec2(TRACK_HEADER_WIDTH, TRACK_TIMELINE_HEIGHT);

        ui.allocate_ui_with_layout(
            desired_size,
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::Frame::central_panel(ui.style())
                    .stroke((0.4, ui.style().noninteractive().fg_stroke.color))
                    .show(ui, |ui| {
                        ui.label(self.track.name_ref());

                        ui.with_layout(egui::Layout::bottom_up(egui::Align::Max), |ui| {
                            ui.button("编辑");
                        });
                    })
            },
        );
    }
}

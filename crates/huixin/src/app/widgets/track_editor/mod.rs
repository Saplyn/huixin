use std::sync::Arc;

use self::{track_header::TrackHeader, track_row::TrackRow};
use crate::{
    app::{helpers::WidgetId, widgets::track_editor::constants::TRACK_HEADER_WIDTH},
    model::{state::CentralState, track::SheetTrackType},
};

mod constants;
mod track_header;
mod track_row;

#[derive(Debug)]
pub struct TrackEditor {
    state: Arc<CentralState>,
}

impl TrackEditor {
    pub fn new(state: Arc<CentralState>) -> Self {
        Self { state }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top(WidgetId::TrackEditorTopPanel)
            .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(emath::vec2(6., 4.)))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        emath::vec2(TRACK_HEADER_WIDTH - 12., 30.),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            if ui
                                .add_sized(
                                    ui.available_size(),
                                    egui::Button::new(egui::RichText::new("添加轨道")),
                                )
                                .clicked()
                            {
                                self.state.sheet_add_track(SheetTrackType::Pattern);
                            };
                        },
                    );

                    ui.separator();
                });
            });

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
                            egui::Frame::NONE.show(ui, |ui| {
                                TrackRow::new(&mut track.value().write(), self.state.clone())
                                    .show(ui);
                            });
                        }
                    });
                });
            });
        });
    }
}

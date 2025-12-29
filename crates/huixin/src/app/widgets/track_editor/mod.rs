use std::sync::Arc;

use egui_dnd::dnd;

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

            let mut tracks_to_delete = Vec::new();
            let mut ordering = self.state.sheet_tracks_ordering_mut();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    dnd(ui, WidgetId::TrackEditorHeaderOrderingDnd).show_vec(
                        &mut ordering,
                        |ui, track_id, handle, _state| {
                            let Some(track) = self.state.sheet_get_track(track_id) else {
                                return;
                            };
                            let output =
                                TrackHeader::new(track_id, &mut track.write(), handle).show(ui);
                            if output.delete_this_track {
                                tracks_to_delete.push(track_id.clone());
                            }
                        },
                    );
                });

                ui.vertical(|ui| {
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        for track_id in ordering.iter() {
                            let Some(track) = self.state.sheet_get_track(track_id) else {
                                continue;
                            };
                            egui::Frame::NONE.show(ui, |ui| {
                                TrackRow::new(&mut track.write(), self.state.clone()).show(ui);
                            });
                        }
                    });
                });
            });
            for track_id in tracks_to_delete {
                self.state.sheet_del_track(&track_id);
            }
        });
    }
}

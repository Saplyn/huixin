use std::fmt::Debug;

use egui_winit::clipboard::Clipboard;
use lyn_util::egui::{copy_color, parse_color};

use crate::model::{
    state::TrackId,
    track::{SheetTrack, SheetTrackTrait},
};

use super::constants::{TRACK_HEADER_WIDTH, TRACK_TIMELINE_HEIGHT};

pub struct TrackHeader<'id, 'track, 'handle> {
    id: &'id TrackId,
    track: &'track mut SheetTrack,
    handle: egui_dnd::Handle<'handle>,
}

impl<'id, 'track, 'handle> Debug for TrackHeader<'id, 'track, 'handle> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackHeader")
            .field("id", self.id)
            .field("track", self.track)
            .finish()
    }
}

#[derive(Debug)]
pub struct TrackHeaderOutput {
    pub delete_this_track: bool,
}

impl<'id, 'track, 'handle> TrackHeader<'id, 'track, 'handle> {
    pub fn new(
        id: &'id TrackId,
        track: &'track mut SheetTrack,
        handle: egui_dnd::Handle<'handle>,
    ) -> Self {
        Self { id, track, handle }
    }
    pub fn show(self, ui: &mut egui::Ui) -> TrackHeaderOutput {
        let mut delete_this_track = false;

        let desired_size = emath::vec2(TRACK_HEADER_WIDTH, TRACK_TIMELINE_HEIGHT);
        let mut editing = ui
            .memory(|mem| mem.data.get_temp::<bool>(egui::Id::new(self.id)))
            .unwrap_or_default();

        ui.horizontal(|ui| {
            let min = ui.max_rect().min;
            let max = emath::pos2(min.x + 10., min.y + TRACK_TIMELINE_HEIGHT);
            let rect = emath::Rect::from_min_max(min, max);
            let painter = ui.painter_at(rect);
            let track_color = self.track.color();
            painter.rect_filled(rect, 0.0, track_color);
            painter.rect_stroke(
                rect,
                0.0,
                (1.0, track_color.lerp_to_gamma(egui::Color32::BLACK, 0.5)),
                egui::StrokeKind::Inside,
            );
            ui.advance_cursor_after_rect(rect);

            ui.allocate_ui_with_layout(
                desired_size,
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::Frame::central_panel(ui.style())
                        .stroke((0.4, ui.style().noninteractive().fg_stroke.color))
                        .show(ui, |ui| {
                            ui.style_mut().spacing.item_spacing = emath::vec2(2., 2.);
                            if editing {
                                ui.horizontal(|ui| {
                                    ui.label("图标：");
                                    ui.add(
                                        egui::TextEdit::singleline(self.track.icon_mut())
                                            .char_limit(2),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label("名称：");
                                    ui.add(egui::TextEdit::singleline(self.track.name_mut()));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("颜色：");
                                    egui::color_picker::color_edit_button_srgba(
                                        ui,
                                        self.track.color_mut(),
                                        egui::color_picker::Alpha::Opaque,
                                    );
                                    if ui.button("󰆏 ").clicked() {
                                        copy_color(self.track.color());
                                    }
                                    if ui.button("󰆒 ").clicked()
                                        && let Some(text) = Clipboard::new(None).get()
                                        && let Some(color) = parse_color(text)
                                    {
                                        *self.track.color_mut() = color;
                                    };
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(self.track.icon_ref());
                                    ui.label(self.track.name_ref());
                                });
                            }

                            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                                ui.horizontal(|ui| {
                                    self.handle.ui(ui, |ui| {
                                        ui.label(egui::RichText::new("󰇜").heading());
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Max),
                                        |ui| {
                                            if editing {
                                                if ui
                                                    .add(
                                                        egui::Button::new(egui::RichText::new(
                                                            " ",
                                                        ))
                                                        .selected(true),
                                                    )
                                                    .clicked()
                                                {
                                                    editing = false;
                                                }
                                                if ui.button(" ").clicked() {
                                                    delete_this_track = true;
                                                }
                                            } else if ui.button(" ").clicked() {
                                                editing = true;
                                            }
                                        },
                                    );
                                });
                            });
                        })
                },
            );
        });

        ui.memory_mut(|mem| {
            mem.data.insert_temp(egui::Id::new(self.id), editing);
        });

        TrackHeaderOutput { delete_this_track }
    }
}

use egui_winit::clipboard::Clipboard;

use crate::{
    app::helpers::{copy_color, parse_color},
    model::{
        state::TrackId,
        track::{SheetTrack, SheetTrackTrait},
    },
};

use super::constants::{TRACK_HEADER_WIDTH, TRACK_TIMELINE_HEIGHT};

#[derive(Debug)]
pub struct TrackHeader<'id, 'track> {
    id: &'id TrackId,
    track: &'track mut SheetTrack,
}

impl<'id, 'track> TrackHeader<'id, 'track> {
    pub fn new(id: &'id TrackId, track: &'track mut SheetTrack) -> Self {
        Self { id, track }
    }
    pub fn show(self, ui: &mut egui::Ui) {
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
                            ui.style_mut().spacing.item_spacing = emath::vec2(4., 4.);
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

                            ui.with_layout(egui::Layout::bottom_up(egui::Align::Max), |ui| {
                                if editing {
                                    if ui
                                        .add(
                                            egui::Button::new(egui::RichText::new(" "))
                                                .selected(true),
                                        )
                                        .clicked()
                                    {
                                        editing = false;
                                    }
                                } else if ui.button(" ").clicked() {
                                    editing = true;
                                }
                            });
                        })
                },
            );
        });

        ui.memory_mut(|mem| {
            mem.data.insert_temp(egui::Id::new(self.id), editing);
        });
    }
}

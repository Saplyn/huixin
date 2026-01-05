use std::{ops::Range, sync::Arc};

use crate::{
    app::widgets::track_editor::constants::TRACK_TIMELINE_HEIGHT,
    model::{
        pattern::{SheetPattern, SheetPatternTrait},
        state::{CentralState, PatternId},
        track::pattern::PatternTrack,
    },
    routines::metronome::TICK_PER_BEAT,
};
use lyn_util::{
    egui::{LynId, text_color},
    types::WithId,
};

pub const RESIZE_HANDLE_WIDTH: f32 = 8.;

#[derive(Debug)]
#[must_use]
pub struct TrackPatternWidget<'track, 'pat> {
    size_per_beat: f32,
    track: &'track mut PatternTrack,
    range: Range<u64>,
    pattern: WithId<(LynId, PatternId), &'pat SheetPattern>,
    tick_snap: u64,
    state: Arc<CentralState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackPatternDragAction {
    Move,
    Resize,
}

#[derive(Debug, Clone, Default)]
struct TrackPatternDragState {
    pub action: Option<TrackPatternDragAction>,
    pub orig_start: u64,
    pub orig_end: u64,
}

impl<'track, 'pat> TrackPatternWidget<'track, 'pat> {
    pub fn new(
        size_per_beat: f32,
        track: &'track mut PatternTrack,
        range: Range<u64>,
        pattern: WithId<(LynId, PatternId), &'pat SheetPattern>,
        tick_snap: u64,
        state: Arc<CentralState>,
    ) -> Self {
        Self {
            size_per_beat,
            track,
            range,
            pattern,
            tick_snap,
            state,
        }
    }

    /// Calculate the rectangle of the pattern based on its range and size.
    fn calc_rect(&self, anchor: egui::Pos2, range: Range<u64>) -> egui::Rect {
        let min = egui::Pos2 {
            x: anchor.x + self.ticks_to_pixels(range.start),
            y: anchor.y,
        };
        let max = egui::Pos2 {
            x: anchor.x + self.ticks_to_pixels(range.end),
            y: anchor.y + TRACK_TIMELINE_HEIGHT,
        };
        egui::Rect::from_min_max(min, max)
    }

    /// Convert ticks to pixels based on the current `size_per_beat`.
    #[inline]
    fn ticks_to_pixels(&self, ticks: u64) -> f32 {
        ticks as f32 / TICK_PER_BEAT as f32 * self.size_per_beat
    }

    /// Convert pixels to ticks based on the current `size_per_beat`.
    #[inline]
    fn pixels_to_ticks(&self, pixels: f32) -> i64 {
        (pixels / self.size_per_beat * TICK_PER_BEAT as f32).round() as i64
    }

    /// Snap ticks to the nearest `tick_snap`.
    #[inline]
    fn snap_ticks(&self, ticks: i64) -> u64 {
        let snap = self.tick_snap as i64;
        ((ticks + snap / 2) / snap * snap).max(0) as u64
    }

    /// Determine which part of the pattern is being interacted with.
    fn hit_test(&self, rect: egui::Rect, pointer: egui::Pos2) -> TrackPatternDragAction {
        let resize_handle = egui::Rect::from_min_max(
            egui::pos2(rect.max.x - RESIZE_HANDLE_WIDTH, rect.min.y),
            rect.max,
        );

        if resize_handle.contains(pointer) {
            TrackPatternDragAction::Resize
        } else {
            TrackPatternDragAction::Move
        }
    }
}

// LYN: Widget Impl

impl<'track, 'pat> TrackPatternWidget<'track, 'pat> {
    pub fn show(mut self, ui: &mut egui::Ui) {
        let anchor = ui.min_rect().left_top();
        let rect = self.calc_rect(anchor, self.range.clone());

        let id = egui::Id::new(self.pattern.id.0);
        let resp = ui.interact(rect, id, egui::Sense::click_and_drag());

        if resp.clicked() {
            self.state.select_pattern(Some(self.pattern.id.1.clone()));
        }

        // Right-click to delete
        if resp.secondary_clicked() {
            let _ = self
                .track
                .del_pattern(self.range.clone(), self.pattern.id.1.clone());
            return;
        }

        let drag_state: TrackPatternDragState = ui.data(|d| d.get_temp(id)).unwrap_or_default();
        let hit_zone = resp.hover_pos().map(|pos| self.hit_test(rect, pos));

        // Update cursor on hover
        if resp.hovered() {
            match hit_zone {
                Some(TrackPatternDragAction::Resize) => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                Some(TrackPatternDragAction::Move) => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }
                None => (),
            }
        }

        // Start drag
        if resp.drag_started() {
            let action = ui.input(|i| i.pointer.press_origin().map(|pos| self.hit_test(rect, pos)));
            let new_drag_state = TrackPatternDragState {
                action,
                orig_start: self.range.start,
                orig_end: self.range.end,
            };
            ui.data_mut(|d| d.insert_temp(id, new_drag_state));
        }

        // Handle dragging
        if resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

            let total_drag = ui.input(|i| {
                i.pointer
                    .press_origin()
                    .map(|origin| i.pointer.interact_pos().unwrap_or(origin) - origin)
                    .unwrap_or(egui::Vec2::ZERO)
            });

            let delta_ticks = self.pixels_to_ticks(total_drag.x);

            let new_range = match drag_state.action {
                Some(TrackPatternDragAction::Move) => {
                    let new_start = self.snap_ticks(drag_state.orig_start as i64 + delta_ticks);
                    let length = drag_state.orig_end - drag_state.orig_start;
                    let new_end = new_start + length;
                    new_start..new_end
                }
                Some(TrackPatternDragAction::Resize) => {
                    let new_end = self
                        .snap_ticks(drag_state.orig_end as i64 + delta_ticks)
                        .max(drag_state.orig_start + self.tick_snap);
                    drag_state.orig_start..new_end
                }
                None => self.range.clone(),
            };

            if new_range != self.range {
                self.track.edit_pattern_range(
                    self.range.clone(),
                    new_range.clone(),
                    self.pattern.id.clone(),
                );
                self.range = new_range;
            }
        }

        // End drag
        if resp.drag_stopped() {
            ui.data_mut(|d| d.remove::<TrackPatternDragState>(id));
        }

        // Draw the pattern
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let painter_rect = ui.painter_at(rect);

            let mut pattern_color = self.pattern.color();
            if !self.pattern.usable() {
                pattern_color = pattern_color.linear_multiply(0.3);
            }
            let stroke_color = if resp.hovered() || resp.dragged() {
                ecolor::Color32::WHITE
            } else {
                pattern_color.lerp_to_gamma(ecolor::Color32::BLACK, 0.5)
            };

            painter.rect(
                rect,
                4.0,
                pattern_color,
                egui::Stroke::new(1.5, stroke_color),
                egui::StrokeKind::Middle,
            );
            painter_rect.text(
                rect.center_top() - egui::vec2(0.0, -10.0),
                egui::Align2::CENTER_CENTER,
                self.pattern.item.name_ref(),
                egui::FontId::default(),
                text_color(pattern_color),
            );

            // Draw resize handles on hover
            if resp.hovered() {
                let handle_color = ecolor::Color32::from_rgba_unmultiplied(255, 255, 255, 100);

                let resize_handle_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.max.x - RESIZE_HANDLE_WIDTH, rect.min.y),
                    rect.max,
                );
                painter.rect_filled(resize_handle_rect, 4.0, handle_color);
            }
        }
    }
}

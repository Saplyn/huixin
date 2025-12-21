use either::Either;

use super::constants::KEY_ROW_HEIGHT;
use crate::{
    model::pattern::midi::{MidiNote, MidiPattern},
    routines::metronome::TICK_PER_BEAT,
};

pub const RESIZE_HANDLE_WIDTH: f32 = 6.;

#[derive(Debug)]
pub struct MidiNoteWidget<'pat> {
    pattern: &'pat mut MidiPattern,
    note: MidiNote,
    id: egui::Id,
    size_per_beat: f32,
    tick_snap: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MidiNoteDragAction {
    Move,
    Resize,
}

#[derive(Debug, Clone, Copy, Default)]
struct MidiNoteDragState {
    pub action: Option<MidiNoteDragAction>,
    pub orig_start: u64,
    pub orig_length: u64,
    pub orig_midicode: u8,
}

impl<'pat> MidiNoteWidget<'pat> {
    pub fn new(
        id: impl std::hash::Hash,
        pattern: &'pat mut MidiPattern,
        note: MidiNote,
        size_per_beat: f32,
        tick_snap: u64,
    ) -> Self {
        Self {
            id: egui::Id::new(id),
            pattern,
            note,
            size_per_beat,
            tick_snap,
        }
    }

    /// Calculate the rectangle of the note based on its `start`, `length`, and `midicode`.
    #[inline]
    fn calc_rect(&self, anchor: egui::Pos2) -> egui::Rect {
        let min = egui::Pos2 {
            x: anchor.x + self.ticks_to_pixels(self.note.start),
            y: anchor.y + (127 - self.note.midicode) as f32 * KEY_ROW_HEIGHT,
        };
        let max = egui::Pos2 {
            x: anchor.x + self.ticks_to_pixels(self.note.start + self.note.length),
            y: anchor.y + (128 - self.note.midicode) as f32 * KEY_ROW_HEIGHT,
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

    /// Determine which part of the note is being interacted with.
    fn hit_test(&self, rect: egui::Rect, pointer: egui::Pos2) -> MidiNoteDragAction {
        let resize_handle = egui::Rect::from_min_max(
            egui::pos2(rect.max.x - RESIZE_HANDLE_WIDTH, rect.min.y),
            rect.max,
        );
        if resize_handle.contains(pointer) {
            MidiNoteDragAction::Resize
        } else {
            MidiNoteDragAction::Move
        }
    }
}

// LYN: Widget Impl

impl<'pat> MidiNoteWidget<'pat> {
    pub fn show(mut self, ui: &mut egui::Ui) {
        let anchor = ui.min_rect().left_top();
        let rect = self.calc_rect(anchor);

        let id = self.id;
        let resp = ui.interact(rect, id, egui::Sense::click_and_drag());

        if resp.secondary_clicked() {
            self.pattern.del_note(Either::Right(self.note));
            return;
        }

        let drag_state: MidiNoteDragState = ui.data(|d| d.get_temp(id)).unwrap_or_default();
        let hit_zone = resp.hover_pos().map(|pos| self.hit_test(rect, pos));

        if resp.hovered() {
            match hit_zone {
                Some(MidiNoteDragAction::Resize) => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                Some(MidiNoteDragAction::Move) => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }
                None => (),
            }
        }

        if resp.drag_started() {
            let action = ui.input(|i| i.pointer.press_origin().map(|pos| self.hit_test(rect, pos)));
            let drag_state = MidiNoteDragState {
                action,
                orig_start: self.note.start,
                orig_length: self.note.length,
                orig_midicode: self.note.midicode,
            };
            ui.data_mut(|d| d.insert_temp(id, drag_state));
        }

        if resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

            let total_drag = ui.input(|i| {
                i.pointer
                    .press_origin()
                    .map(|origin| i.pointer.interact_pos().unwrap_or(origin) - origin)
                    .unwrap_or(egui::Vec2::ZERO)
            });

            match drag_state.action {
                Some(MidiNoteDragAction::Move) => {
                    let delta_ticks = self.pixels_to_ticks(total_drag.x);
                    let new_start = self.snap_ticks(drag_state.orig_start as i64 + delta_ticks);

                    let row_delta = (total_drag.y / KEY_ROW_HEIGHT).round() as i16;
                    let new_midicode =
                        (drag_state.orig_midicode as i16 - row_delta).clamp(0, 127) as u8;

                    self.pattern.edit_note(Either::Right(self.note), |n| {
                        n.start = new_start;
                        n.midicode = new_midicode;
                    });
                    self.note.start = new_start;
                    self.note.midicode = new_midicode;
                }
                Some(MidiNoteDragAction::Resize) => {
                    let delta_ticks = self.pixels_to_ticks(total_drag.x);
                    let raw = drag_state.orig_length as i64 + delta_ticks;
                    let new_length = self.snap_ticks(raw.max(self.tick_snap as i64));
                    if new_length > 0 {
                        self.pattern.edit_note(Either::Right(self.note), |n| {
                            n.length = new_length;
                        });
                        self.note.length = new_length;
                    }
                }
                None => (),
            }
        }

        if resp.drag_stopped() {
            ui.data_mut(|d| d.remove::<MidiNoteDragState>(id));
        }

        if ui.is_rect_visible(rect) {
            let note_color = ecolor::Color32::from_rgb(100, 149, 237);
            let stroke_color = if resp.hovered() || resp.dragged() {
                ecolor::Color32::WHITE
            } else {
                ecolor::Color32::from_rgb(70, 100, 170)
            };

            ui.painter().rect(
                rect,
                2.0,
                note_color,
                egui::Stroke::new(1.0, stroke_color),
                egui::StrokeKind::Middle,
            );

            if resp.hovered() {
                let handle_color = ecolor::Color32::from_rgba_unmultiplied(255, 255, 255, 100);

                let right_handle_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.max.x - RESIZE_HANDLE_WIDTH, rect.min.y),
                    rect.max,
                );
                ui.painter()
                    .rect_filled(right_handle_rect, 2.0, handle_color);
            }
        }
    }
}

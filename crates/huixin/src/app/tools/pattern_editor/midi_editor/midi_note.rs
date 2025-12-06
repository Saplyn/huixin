#[derive(Debug)]
pub struct MidiNote;

impl egui::Widget for MidiNote {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, resp) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 0.0, ecolor::Color32::RED);
        }

        resp
    }
}

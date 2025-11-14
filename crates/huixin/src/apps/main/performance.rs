// REF: https://github.com/emilk/egui/blob/0a81372cfd3a4deda640acdecbbaf24bf78bb6a2/crates/egui_demo_app/src/frame_history.rs

use egui::util::History;

#[derive(Debug)]
pub struct Performance {
    frame_times: History<f32>,
}

impl Default for Performance {
    fn default() -> Self {
        let max_age: f32 = 1.0;
        let max_len = (max_age * 300.0).round() as usize;
        Self {
            frame_times: History::new(0..max_len, max_age),
        }
    }
}

impl Performance {
    pub fn update_frame_history(&mut self, now: f64, previous_frame_time: Option<f32>) {
        let previous_frame_time = previous_frame_time.unwrap_or_default();
        if let Some(latest) = self.frame_times.latest_mut() {
            *latest = previous_frame_time; // rewrite history now that we know
        }
        self.frame_times.add(now, previous_frame_time); // projected
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!(
            "Mean CPU usage: {:.2} ms / frame",
            1e3 * self.frame_times.average().unwrap_or_default()
        ));
    }
}

use self::{helpers::WidgetId, widgets::performance::Performance};

mod helpers;
mod widgets;

// LYN: Main App State Holder

#[derive(Debug)]
pub struct MainApp {
    // widget states
    pub performance: Performance,
}

impl MainApp {
    pub fn prepare() -> Self {
        Self {
            performance: Default::default(),
        }
    }
}

// LYN: Main App UI Implementation

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.performance
            .update_frame_history(ctx.input(|i| i.time), frame.info().cpu_usage);

        self.draw_ui(ctx);
        ctx.request_repaint(); // Uncomment this for continuous repainting (fix some UI update issues)
    }
}

impl MainApp {
    fn draw_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top(WidgetId::MainAppTopToolBar).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("left");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("right");
                    ui.separator();
                });
            })
        });

        egui::TopBottomPanel::bottom(WidgetId::MainAppButtonStatusBar).show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.performance.ui(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(env!("BUILD_INFO"));
                });
            });
        });

        egui::SidePanel::left(WidgetId::MainAppLeftExplorerPanel)
            .show(ctx, |ui| ui.label("Explorer"));

        egui::CentralPanel::default().show(ctx, |ui| ui.label("Canvas"));
    }
}

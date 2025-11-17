use crate::app::tools::ToolWindow;

#[derive(Debug)]
pub struct Tester {
    open: bool,
}

impl Default for Tester {
    fn default() -> Self {
        Self { open: true }
    }
}

impl ToolWindow for Tester {
    fn icon(&self) -> String {
        "󰄛 ".to_string()
    }

    fn window_open(&self) -> bool {
        self.open
    }

    fn window_open_mut(&mut self) -> &mut bool {
        &mut self.open
    }

    fn toggle_open(&mut self, open: Option<bool>) {
        if let Some(open) = open {
            self.open = open;
        } else {
            self.open = !self.open;
        }
    }

    fn draw(&mut self, ctx: &egui::Context) {
        egui::Window::new("Test Tool Window 测试工具窗口")
            .id(egui::Id::new("test-tool-window"))
            .frame(egui::Frame::window(&ctx.style()).inner_margin(0))
            .title_bar(false)
            .min_size(emath::vec2(300., 150.))
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("tester-top").show_inside(ui, |ui| {
                    ui.label("top");
                });

                egui::SidePanel::right("tester-right")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.label("right");
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.label("Meow meow 喵喵 󰄛 ");
                    ui.code("Meow meow 喵喵 󰄛 ");
                });
            });
    }
}

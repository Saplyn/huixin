use crate::apps::helpers::WidgetId;

#[derive(Debug, Default)]
pub struct ErrorModal {
    pub msg: Option<String>,
}

impl ErrorModal {
    pub fn set_msg(&mut self, msg: String) {
        self.msg = Some(msg);
    }

    pub fn clear_msg(&mut self) {
        self.msg = None;
    }

    pub fn try_draw(&mut self, ctx: &egui::Context) {
        if let Some(msg) = &self.msg {
            egui::Modal::new(WidgetId::ErrorModal.into()).show(ctx, |ui| {
                ui.label("CRITICAL APP ERROR");
                ui.label(msg);
            });
        }
    }
}

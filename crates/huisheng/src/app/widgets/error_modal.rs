use crate::app::helpers::WidgetId;

#[derive(Debug, Default)]
pub struct ErrorModal<'msg> {
    pub msg: &'msg str,
}

impl<'msg> ErrorModal<'msg> {
    pub fn new(msg: &'msg str) -> Self {
        Self { msg }
    }
    pub fn draw(&mut self, ctx: &egui::Context) {
        egui::Modal::new(WidgetId::ErrorModal.into()).show(ctx, |ui| {
            ui.label(egui::RichText::new("重大错误").heading().strong());
            ui.label(self.msg);
        });
    }
}

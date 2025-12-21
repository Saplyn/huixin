use lyn_util::egui::EguiContextExt;
use simplelog::TermLogger;

use crate::app::MainApp;

mod app;
mod model;
mod routines;

const APP_ID: &str = "huixin";

fn main() -> eframe::Result {
    init_logger().expect("Fail to start logger");

    let mut app = MainApp::init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        APP_ID,
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.load_chinese_fonts();
            app.prepare_launch(cc);
            Ok(Box::new(app))
        }),
    )
}

fn init_logger() -> Result<(), log::SetLoggerError> {
    TermLogger::init(
        log::LevelFilter::Trace,
        simplelog::ConfigBuilder::new()
            .add_filter_allow_str(env!("CARGO_PKG_NAME"))
            .build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
}

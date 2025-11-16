use lyn_util::EguiContextExt;
use simplelog::TermLogger;

use crate::app_ui::MainApp;

mod app_ui;
mod routines;
mod sheet;

fn main() -> eframe::Result {
    init_logger().expect("Fail to start logger");

    let app = MainApp::prepare();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Huixin",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.load_chinese_fonts();
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

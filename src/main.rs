use std::thread;

use simplelog::TermLogger;

use crate::app::App;

mod app;
mod metronome;
mod ui;

fn main() -> eframe::Result {
    init_logger().expect("Fail to start logger");

    let app = App::new();
    let metronome = app.metronome.clone();
    thread::spawn(move || metronome::main(metronome));

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Unnamed App",
        native_options,
        Box::new(|cc| Ok(Box::new(app))),
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

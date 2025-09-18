use std::{
    sync::{Arc, mpsc},
    thread,
};

use crate::{
    metronome::{self, Metronome},
    ui::{
        composer::Composer, main::UI, networker::Networker, programmer::Programmer, tester::Tester,
    },
};

#[derive(Debug)]
pub struct App {
    pub ui: UI,
    pub metronome: Arc<Metronome>,
}

pub fn prepare_app() -> App {
    let (ui_cmd_tx, ui_cmd_rx) = mpsc::channel();
    let app = App {
        ui: UI {
            pages: vec![
                Box::new(Tester),
                Box::new(Composer),
                Box::new(Programmer),
                Box::new(Networker),
            ],
            cmd_tx: ui_cmd_tx.clone(),
            cmd_rx: ui_cmd_rx,
            active_page: Default::default(),
            performance: Default::default(),
            error_modal: Default::default(),
        },
        metronome: Arc::new(Metronome::new()),
    };

    let metronome = app.metronome.clone();
    thread::spawn(move || metronome::main(metronome, ui_cmd_tx));

    app
}

use std::{
    sync::{Arc, mpsc},
    thread,
};

use crate::{
    metronome::{self, Metronome},
    sheet_reader::{self, SheetReader},
    ui::{
        composer::Composer, main::UI, networker::Networker, programmer::Programmer, tester::Tester,
    },
};

#[derive(Debug)]
pub struct App {
    pub ui: UI,
    pub metronome: Arc<Metronome>,
}

impl App {
    pub fn prepare() -> App {
        let (ui_cmd_tx, ui_cmd_rx) = mpsc::channel();
        let metronome = Arc::new(Metronome::new());
        let sheet_reader = Arc::new(SheetReader::new());

        thread::spawn({
            let state = metronome.clone();
            let ui_cmd_tx = ui_cmd_tx.clone();
            move || metronome::main(state, ui_cmd_tx)
        });
        thread::spawn({
            let state = sheet_reader.clone();
            let metro = metronome.clone();
            let ui_cmd_tx = ui_cmd_tx.clone();
            move || sheet_reader::main(state, metro, ui_cmd_tx)
        });

        Self {
            ui: UI {
                pages: vec![
                    Box::new(Tester {}),
                    Box::new(Composer { sheet_reader }),
                    Box::new(Programmer {}),
                    Box::new(Networker {}),
                ],
                cmd_tx: ui_cmd_tx,
                cmd_rx: ui_cmd_rx,
                active_page: Default::default(),
                performance: Default::default(),
                error_modal: Default::default(),
            },
            metronome,
        }
    }
}

use std::sync::Arc;

use crate::{
    metronome::Metronome,
    ui::{
        composer::Composer, main::UI, networker::Networker, programmer::Programmer, tester::Tester,
    },
};

#[derive(Debug, Default)]
pub struct App {
    pub ui: UI,
    pub metronome: Arc<Metronome>,
}

impl App {
    pub fn new() -> Self {
        Self {
            ui: UI {
                pages: vec![
                    Box::new(Tester),
                    Box::new(Composer),
                    Box::new(Programmer),
                    Box::new(Networker),
                ],
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

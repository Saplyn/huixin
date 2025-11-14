use crate::sheet::Timed;

// LYN: Pattern

#[derive(Debug)]
pub struct SheetPattern {
    pub name: String,
    pub inner: SheetPatternInner,
}

#[derive(Debug)]
pub enum SheetPatternInner {
    Piano(PianoPattern),
    // Curve(CurvePattern),
    // Event(EventPattern),
}

// LYN: Pattern - Piano

#[derive(Debug)]
pub struct PianoPattern {
    pub notes: Vec<Timed<PianoNote>>,
}

#[derive(Debug)]
pub struct PianoNote {
    pub strength: u16,
    pub code: u8,
    pub length: u64,
}

impl PianoPattern {
    fn get_action(&self, tick: u16) {}
}

// LYN: Pattern - Curve

// #[derive(Debug)]
// pub struct CurvePattern {
//     vals: Vec<Timed<f64>>,
// }

// LYN: Pattern - Event

// #[derive(Debug)]
// pub struct EventPattern {}

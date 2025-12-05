use crate::sheet::Timed;

// LYN: Pattern

#[derive(Debug)]
pub struct SheetPattern {
    pub name: String,
    pub inner: SheetPatternInner,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SheetPatternType {
    Midi,
    // Curve,
    // Event,
}

#[derive(Debug)]
pub enum SheetPatternInner {
    Midi(MidiPattern),
    // Curve(CurvePattern),
    // Event(EventPattern),
}

// LYN: Pattern - Midi

#[derive(Debug)]
pub struct MidiPattern {
    pub notes: Vec<Timed<MidiNote>>,
}

#[derive(Debug)]
pub struct MidiNote {
    pub midicode: u8,
    pub strength: u16,
    pub length: u64,
}

impl MidiPattern {
    pub fn notes_ref(&self) -> &Vec<Timed<MidiNote>> {
        &self.notes
    }

    pub fn notes_mut(&mut self) -> &mut Vec<Timed<MidiNote>> {
        &mut self.notes
    }
}

// LYN: Pattern - Curve

// #[derive(Debug)]
// pub struct CurvePattern {
//     vals: Vec<Timed<f64>>,
// }

// LYN: Pattern - Event

// #[derive(Debug)]
// pub struct EventPattern {}

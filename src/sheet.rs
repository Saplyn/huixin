use std::sync::Weak;

// LYN: Timed

#[derive(Debug)]
pub struct Timed<T> {
    tick: u64,
    inner: T,
}

#[allow(unused)] // TODO:
impl<T> Timed<T> {
    pub fn flatten(self) -> (u64, T) {
        (self.tick, self.inner)
    }
    pub fn flatten_ref(&self) -> (u64, &T) {
        (self.tick, &self.inner)
    }
    pub fn flatten_mut(&mut self) -> (u64, &mut T) {
        (self.tick, &mut self.inner)
    }
}

// LYN: Track

#[derive(Debug)]
pub struct SheetTrack {
    pattern: Timed<Weak<SheetPattern>>,
}

// LYN: Pattern

#[derive(Debug)]
pub enum SheetPattern {
    Piano(PianoPattern),
    Curve(CurvePattern),
    Event(EventPattern),
}

// LYN: Pattern - Piano

#[derive(Debug)]
pub struct PianoPattern {
    notes: Vec<Timed<PianoNote>>,
}

#[derive(Debug)]
pub struct PianoNote {}

// LYN: Pattern - Curve

#[derive(Debug)]
pub struct CurvePattern {
    vals: Vec<Timed<f64>>,
}

// LYN: Pattern - Event

#[derive(Debug)]
pub struct EventPattern {}

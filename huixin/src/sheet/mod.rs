use std::sync::Weak;

use crate::sheet::pattern::SheetPattern;

pub mod pattern;

// LYN: Timed

#[derive(Debug)]
pub struct Timed<T> {
    pub tick: u64,
    pub inner: T,
}

#[allow(unused)] // TODO:
impl<T> Timed<T> {
    pub fn new(tick: u64, inner: T) -> Self {
        Self { tick, inner }
    }
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
    pub name: String,
    pub pattern: Vec<Timed<Weak<SheetPattern>>>,
}

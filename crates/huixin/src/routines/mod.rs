pub mod guardian;
pub mod instructor;
pub mod metronome;
pub mod sheet_reader;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RoutineId {
    Main,
    Guardian,
    Instructor,
    Metronome,
    SheetReader,
}

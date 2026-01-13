pub mod guardian;
pub mod listener;
pub mod processor;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RoutineId {
    Processor,
    Listener,
}

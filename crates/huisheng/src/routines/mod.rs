pub mod guardian;
pub mod processor;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RoutineId {
    Processor,
}

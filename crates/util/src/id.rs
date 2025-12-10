use std::sync::atomic::{AtomicU64, Ordering};

const ID_HASH_PREFIX: &str = "LynId";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LynId(u64);

impl LynId {
    pub fn obtain_id() -> LynId {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        LynId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::hash::Hash for LynId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (ID_HASH_PREFIX, self.0).hash(state);
    }
}

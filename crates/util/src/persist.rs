// LYN: EFrame Storage

pub struct AppStore;

impl AppStore {
    const STORAGE_PREFIX: &str = "lyn";

    pub fn key(key: impl AsRef<str>) -> String {
        format!("{}:{}", Self::STORAGE_PREFIX, key.as_ref())
    }
}

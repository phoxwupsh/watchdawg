use dashmap::DashMap;
use super::SessionStore;

pub struct MemoryStore {
    inner: DashMap<String, u64>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }
}

impl SessionStore for MemoryStore {
    fn load(&self, session_id: &str) -> Option<u64> {
        self.inner.get(session_id).map(|res| *res.value())
    }
    fn save(&self, session_id: &str, timestamp: u64) -> Option<()> {
        self.inner.insert(session_id.to_string(), timestamp)?;
        Some(())
    }
    fn delete(&self, session_id: &str) -> Option<u64> {
        self.inner.remove(session_id).map(|(_key, value)| value)
    }
}
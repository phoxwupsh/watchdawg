use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub mod memory;
pub mod redis;

pub trait SessionStore {
    fn load(&self, session_id: &str) -> Option<u64>;
    fn save(&self, session_id: &str, timestamp: u64) -> Option<()>;
    fn delete(&self, session_id: &str) -> Option<u64>;
}

pub struct SessionManager {
    pub cookie_name: String,
    pub max_age: u64,
    store: Arc<dyn SessionStore + Send + Sync>,
}

impl SessionManager {
    pub fn new(
        cookie_name: impl Into<String>,
        store: Arc<dyn SessionStore + Send + Sync>,
        max_age: u64,
    ) -> Self {
        Self {
            cookie_name: cookie_name.into(),
            store,
            max_age,
        }
    }

    pub fn create_session(&self) -> String {
        let uuid = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.store.save(&uuid, now);
        uuid
    }

    pub fn is_session_valid(&self, session: &str) -> bool {
        let Some(since) = self.store.load(session) else {
            return false;
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let res = now.saturating_sub(since) < self.max_age;

        // delete session if expired
        if !res {
            self.store.delete(session);
        }
        res
    }
}

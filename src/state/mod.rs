use crate::config::{Database, RedisStore};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: RedisStore,
}

impl AppState {
    pub fn new(db: Database, redis: RedisStore) -> Self {
        Self { db, redis }
    }
}

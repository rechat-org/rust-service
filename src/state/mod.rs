use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::{Database, RedisStore, StripeClient};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: RedisStore,
    pub stripe: StripeClient,
    pub active_users: Arc<RwLock<i64>>  // This is now tokio's RwLock
}

impl AppState {
    pub fn new(db: Database, redis: RedisStore, stripe: StripeClient) -> Self {
        Self {
            db,
            redis,
            stripe,
            active_users: Arc::new(RwLock::new(0)),
        }
    }
}

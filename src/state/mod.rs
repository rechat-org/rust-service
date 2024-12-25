use crate::config::{Database, RedisStore, StripeClient};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: RedisStore,
    pub stripe: StripeClient,
}

impl AppState {
    pub fn new(db: Database, redis: RedisStore, stripe: StripeClient) -> Self {
        Self { db, redis, stripe }
    }
}

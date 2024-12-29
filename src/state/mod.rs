use crate::config::{Database, StripeClient};
use chrono::Timelike;
use redis::{AsyncCommands, RedisError};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: RedisStore,
    pub stripe: StripeClient,
}

#[derive(Clone)]
pub struct RedisStore {
    client: redis::Client,
}

impl RedisStore {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    /// Get current usage for an organization
    pub async fn get_usage(&self, org_id: &Uuid) -> Result<Option<i64>, RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("usage:{}", org_id);

        let result: Option<i64> = conn.get(&key).await?;
        Ok(result)
    }

    /// Increment usage for an organization
    /// Returns the new value after incrementing
    pub async fn increment_usage(&self, org_id: &Uuid) -> Result<i64, RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("usage:{}", org_id);

        // Increment and reset expiry
        let new_value: i64 = conn.incr(&key, 1).await?;

        // Reset TTL to end of current month
        let ttl = Self::get_ttl_until_month_end();
        let _: () = conn.expire(&key, ttl as i64).await?;

        Ok(new_value)
    }

    /// Set usage for an organization with TTL until end of month
    pub async fn set_usage(&self, org_id: &Uuid, usage: i64) -> Result<(), RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("usage:{}", org_id);

        // Set value with TTL until end of month
        let ttl = Self::get_ttl_until_month_end();
        let _: () = conn.set_ex(&key, usage, ttl as usize as u64).await?;

        Ok(())
    }

    fn get_ttl_until_month_end() -> u64 {
        use chrono::{Datelike, Utc};

        let now = Utc::now();
        let next_month = if now.month() == 12 {
            // For December, go to next year January
            Utc::now()
                .with_year(now.year() + 1)
                .unwrap()
                .with_month(1)
                .unwrap()
                .with_day(1)
                .unwrap()
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
        } else {
            // For other months, just go to next month
            Utc::now()
                .with_month(now.month() + 1)
                .unwrap()
                .with_day(1)
                .unwrap()
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
        };

        (next_month - now).num_seconds() as u64
    }
}

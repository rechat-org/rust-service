use redis::{AsyncCommands, Client, RedisError, RedisResult};
use std::env;
use chrono::Timelike;
use uuid::Uuid;

#[derive(Clone)]
pub struct RedisConfig {
    pub url: String,
}

impl RedisConfig {
    pub fn new() -> Self {
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        Self { url: redis_url }
    }

    pub fn create_client(&self) -> RedisResult<Client> {
        Client::open(self.url.clone())
    }
}

#[derive(Clone)]
pub struct RedisStore {
    pub client: Client,
}

impl RedisStore {
    pub fn new(config: RedisConfig) -> redis::RedisResult<Self> {
        let client = config.create_client()?;
        Ok(Self { client })
    }

    pub async fn ping(&self) -> redis::RedisResult<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map(|response: String| response == "PONG")
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

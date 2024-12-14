use redis::{Client, RedisResult};
use std::env;

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
        let mut conn = self.client.get_async_connection().await?;
        redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map(|response: String| response == "PONG")
    }
}

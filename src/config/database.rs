use sea_orm::*;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Database {
    pub connection: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Result<Self, DbErr> {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true);

        // Fixed: using .connect() instead of .connect_with_options()
        let connection = sea_orm::Database::connect(opt).await?;

        Ok(Self { connection })
    }

    pub async fn ping(&self) -> Result<(), DbErr> {
        self.connection.ping().await
    }
}
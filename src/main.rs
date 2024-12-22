mod config;
mod handlers;
mod router;
mod state;
mod entities;
mod utils;
mod middleware;

use config::{AppConfig, Database, RedisConfig, RedisStore};
use router::api_router;
use state::AppState;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    let app_config = AppConfig::new();

    let database = Database::new()
        .await
        .expect("Failed to connect to database");

    // Verify database connection
    database.ping().await.expect("Could not ping database");

    // Initialize Redis
    let redis_config = RedisConfig::new();
    let redis_store = RedisStore::new(redis_config).expect("Failed to create Redis client.");

    // Verify Redis connection
    redis_store.ping().await.expect("Could not ping Redis");

    // Create app state
    let state = AppState::new(database, redis_store);

    // Build our application with routes
    let app = api_router().with_state(state);

    let addr = app_config.addr();
    println!("🚀 Server running on http://{}", addr);

    // Create and start the server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Start the server
    axum::serve(listener, app).await.unwrap();
}

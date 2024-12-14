mod config;
mod handlers;
mod router;
mod state;

use axum::{extract::State};
use config::{AppConfig, Database};
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

    // Create app state
    let state = AppState::new(database);

    // Build our application with routes
    let app = api_router().with_state(state);

    let addr = app_config.addr();
    println!("🚀 Server running on http://{}", addr);

    // Create and start the server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Start the server
    axum::serve(listener, app).await.unwrap();
}

// Health check endpoint
async fn health_check(State(state): State<AppState>) -> String {
    match state.db.ping().await {
        Ok(_) => "🟢 Database connected!".to_string(),
        Err(e) => format!("🔴 Database error: {}", e),
    }
}

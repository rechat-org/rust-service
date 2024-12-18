use crate::state::AppState;
use axum::extract::State;

pub async fn health_check(State(state): State<AppState>) -> String {
    tracing::info!("Health check endpoint called");
    match state.db.ping().await {
        Ok(_) => {
            tracing::info!("Database ping successful");
            "🟢 Database connected 🟢".to_string()
        }
        Err(e) => {
            tracing::error!("Database ping failed: {}", e);
            format!("🔴 Database error: {}", e)
        }
    }
}

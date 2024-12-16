use crate::state::AppState;
use axum::extract::{Path, State};

pub async fn create_channel(State(state): State<AppState>) {
    tracing::info!("Health check endpoint called");
}

pub async fn get_channel_by_id(state: State<AppState>, Path(room_id): Path<String>) {}

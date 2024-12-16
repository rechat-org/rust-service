use crate::entities::{channel, channel::Entity as Channel, prelude::*};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateChannelResponse {
    id: Uuid,
    name: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    message: String,
}

pub async fn create_channel(
    State(state): State<AppState>,
    Json(payload): Json<CreateChannelRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_channel");

    let db = &state.db.connection;
    let name = payload.name;

    let new_channel = channel::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name.clone()),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
    };

    match Channel::insert(new_channel).exec(db).await {
        Ok(channel) => {
            let response = CreateChannelResponse {
                id: channel.last_insert_id,
                name,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(err) => {
            tracing::error!("Failed to create channel: {:?}", err);
            let error_response = ErrorResponse {
                message: "Failed to create channel".to_string(), // Fixed error message
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

pub async fn get_channel_by_id(state: State<AppState>, Path(room_id): Path<String>) {}

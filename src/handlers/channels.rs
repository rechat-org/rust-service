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

pub async fn get_channel_by_id(
    state: State<AppState>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    let db = &state.db.connection;

    let channel_id = match Uuid::parse_str(&channel_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    message: "Invalid UUID format".to_string(),
                }),
            )
                .into_response();
        }
    };

    match Channel::find_by_id(channel_id).one(db).await {
        Ok(Some(channel)) => {
            let response = CreateChannelResponse {
                id: channel.id,
                name: channel.name,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                message: "Channel not found".to_string(),
            }),
        )
            .into_response(),
        Err(err) => {
            tracing::error!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    message: "Failed to fetch channel".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn get_channels(state: State<AppState>) -> impl IntoResponse {
    let db = &state.db.connection;

    match Channel::find().all(db).await {
        Ok(channels) => {
            let response = channels
                .into_iter()
                .map(|channel| CreateChannelResponse {
                    id: channel.id,
                    name: channel.name,
                })
                .collect::<Vec<_>>();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(err) => {
            tracing::error!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    message: "Failed to fetch channels".to_string(),
                }),
            )
                .into_response()
        }
    }
}

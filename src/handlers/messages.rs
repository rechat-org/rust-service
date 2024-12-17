use crate::entities::{messages, prelude::*};
use axum::{extract::Path, Json};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CreateMessageResponse {
    id: Uuid,
    content: String,
    participant_id: Uuid,
    channel_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    content: String,
    participant_id: Uuid,
    channel_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    message: String,
}

pub async fn create_message(
    State(state): State<AppState>,
    Json(payload): Json<CreateMessageRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_participant");

    let db = &state.db.connection;

    let content = payload.content;
    let participant_id = payload.participant_id;
    let channel_id = payload.channel_id;

    let new_message = messages::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(content.clone()),
        participant_id: Set(participant_id),
        channel_id: Set(channel_id),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
    };

    match Messages::insert(new_message)
        .exec(db).await {
        Ok(message) => {
            let response = CreateMessageResponse {
                id: message.last_insert_id,
                content,
                participant_id,
                channel_id,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(err) => {
            tracing::error!("Failed to create participant: {:?}", err);
            let error_response = ErrorResponse {
                message: "Failed to create participant".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

pub async fn get_messages_by_channel_id(
    State(state): State<AppState>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("executes: create_participant");

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

    match Messages::find()
        .filter(messages::Column::ChannelId.eq(channel_id))
        .order_by_asc(messages::Column::CreatedAt)
        .all(db)
        .await
    {
        Ok(messages) => {
            let messages = messages
                .into_iter()
                .map(|message| CreateMessageResponse {
                    id: message.id,
                    content: message.content,
                    participant_id: message.participant_id,
                    channel_id: message.channel_id,
                })
                .collect::<Vec<_>>();
            (StatusCode::OK, Json(messages)).into_response()
        }
        Err(err) => {
            tracing::error!("Failed to get messages: {:?}", err);
            let error_response = ErrorResponse {
                message: "Failed to get messages".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

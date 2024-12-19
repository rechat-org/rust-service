use crate::entities::{channel, messages, prelude::*};
use crate::utils::ServerResponse;
use axum::{extract::Path, Json};
use axum::{extract::State, response::IntoResponse};
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
    channel_name: String,
}

pub async fn create_message(
    State(state): State<AppState>,
    Json(payload): Json<CreateMessageRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_participant");

    let db = &state.db.connection;

    let content = payload.content;
    let participant_id = payload.participant_id;
    let channel_name = payload.channel_name;

    let channel = match Channel::find()
        .filter(channel::Column::Name.eq(channel_name))
        .one(db)
        .await
    {
        Ok(Some(channel)) => channel,
        Ok(None) => return ServerResponse::bad_request("Channel not found"),
        Err(err) => return ServerResponse::server_error(err, "Failed to check if channel exists"),
    };

    let new_message = messages::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(content.clone()),
        participant_id: Set(participant_id),
        channel_id: Set(channel.id),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
    };

    match Messages::insert(new_message).exec(db).await {
        Ok(message) => {
            let response = CreateMessageResponse {
                id: message.last_insert_id,
                content,
                participant_id,
                channel_id: channel.id,
            };
            ServerResponse::created(response)
        }
        Err(err) => ServerResponse::server_error(err, "Failed to create message"),
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
        Err(_) => return ServerResponse::bad_request("Invalid channel ID"),
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
            ServerResponse::ok(messages)
        }
        Err(err) => ServerResponse::server_error(err, "Failed to fetch messages"),
    }
}

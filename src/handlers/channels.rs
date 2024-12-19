use crate::entities::{channel, channel::Entity as Channel};
use crate::state::AppState;
use crate::utils::{ServerResponse};
use axum::{
    extract::{Path, State},
    response::{IntoResponse},
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


pub async fn create_channel(
    State(state): State<AppState>,
    Json(payload): Json<CreateChannelRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_channel");

    let db = &state.db.connection;
    let name = payload.name;

    // checks if channel name exist
    match Channel::find()
        .filter(channel::Column::Name.contains(name.clone()))
        .one(db)
        .await
    {
        Ok(Some(_)) => {
            return ServerResponse::bad_request("Channel with this name already exists");
        }
        Ok(None) => {}
        Err(err) => {
            return ServerResponse::server_error(err, "Failed to check if channel name exists");
        }
    }

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
            ServerResponse::ok(response)
        }
        Err(err) => ServerResponse::server_error(err, "Failed to create channel"),
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
            return ServerResponse::bad_request("Invalid channel ID");
        }
    };

    match Channel::find_by_id(channel_id).one(db).await {
        Ok(Some(channel)) => {
            let response = CreateChannelResponse {
                id: channel.id,
                name: channel.name,
            };
            ServerResponse::ok(response)
        }
        Ok(None) => ServerResponse::not_found("Channel not found"),
        Err(err) => ServerResponse::server_error(err, "Failed to fetch channel"),
    }
}

pub async fn get_channels(state: State<AppState>) -> impl IntoResponse {
    let db = &state.db.connection;

    match Channel::find()
        .order_by_desc(channel::Column::CreatedAt)
        .all(db)
        .await
    {
        Ok(channels) => {
            let response = channels
                .into_iter()
                .map(|channel| CreateChannelResponse {
                    id: channel.id,
                    name: channel.name,
                })
                .collect::<Vec<_>>();
            ServerResponse::ok(response)
        }
        Err(err) => ServerResponse::server_error(err, "Failed to fetch channels"),
    }
}

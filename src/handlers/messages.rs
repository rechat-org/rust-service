use crate::entities::{channels, messages, organization_members, prelude::*};
use crate::middleware::api_key_authorizer::ApiKeyAuthorizer;
use crate::middleware::authorization::AuthorizedOrganizationUser;
use crate::middleware::usage_limiter::UsageLimiter;
use crate::middleware::usage_tracker::UsageTracker;
use crate::state::AppState;
use crate::utils::ServerResponse;
use axum::{extract::Path, Json};
use axum::{extract::State, response::IntoResponse};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Datelike, Timelike, Utc};

#[derive(Debug, Serialize)]
pub struct CreateMessageResponse {
    id: Uuid,
    content: String,
    participant_id: Uuid,
    channel_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct CountMessageResponse {
    count: u32,
    month: u8,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    content: String,
    participant_id: Uuid,
    channel_name: String,
}

pub async fn create_message(
    State(state): State<AppState>,
    _: ApiKeyAuthorizer,
    _: UsageTracker,
    _: UsageLimiter,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> impl IntoResponse {
    let db = &state.db.connection;

    let channel = match Channels::find()
        .filter(channels::Column::Name.eq(&payload.channel_name))
        .filter(channels::Column::OrganizationId.eq(org_id))
        .one(db)
        .await
    {
        Ok(Some(channel)) => channel,
        Ok(None) => return ServerResponse::bad_request("Channel not found in this organization"),
        Err(err) => return ServerResponse::server_error(err, "Failed to check channel"),
    };

    let new_message = messages::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(payload.content.clone()),
        participant_id: Set(payload.participant_id),
        channel_id: Set(channel.id),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
    };

    match Messages::insert(new_message).exec(db).await {
        Ok(message) => {
            let response = CreateMessageResponse {
                id: message.last_insert_id,
                content: payload.content,
                participant_id: payload.participant_id,
                channel_id: channel.id,
            };
            ServerResponse::created(response)
        }
        Err(err) => ServerResponse::server_error(err, "Failed to create message"),
    }
}

pub async fn get_messages_by_channel_id(
    State(state): State<AppState>,
    _: ApiKeyAuthorizer,
    _: UsageLimiter,
    Path((org_id, channel_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let db = &state.db.connection;
    let channel_id = match Uuid::parse_str(&channel_id) {
        Ok(id) => id,
        Err(_) => return ServerResponse::bad_request("Invalid channel ID"),
    };
    let org_id = match Uuid::parse_str(&org_id) {
        Ok(id) => id,
        Err(_) => return ServerResponse::bad_request("Invalid organization ID"),
    };

    // First verify channel belongs to organization
    let channel = match Channels::find()
        .filter(channels::Column::Id.eq(channel_id))
        .filter(channels::Column::OrganizationId.eq(org_id))
        .one(db)
        .await
    {
        Ok(Some(channel)) => channel,
        Ok(None) => return ServerResponse::forbidden("Channel not found in this organization"),
        Err(err) => return ServerResponse::server_error(err, "Failed to verify channel"),
    };

    match Messages::find()
        .filter(messages::Column::ChannelId.eq(channel.id))
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


pub async fn get_messages_count_for_current_month(
    State(state): State<AppState>,
    auth: AuthorizedOrganizationUser,
) -> impl IntoResponse {
    tracing::info!("executes: get_messages_count_for_current_month");

    let now = Utc::now();
    let start_of_month = Utc::now()
        .with_day(1)
        .unwrap()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    let end_of_month = if now.month() == 12 {
        Utc::now()
            .with_year(now.year() + 1)
            .unwrap()
            .with_month(1)
            .unwrap()
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    } else {
        Utc::now()
            .with_month(now.month() + 1)
            .unwrap()
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    };

    let db = &state.db.connection;
    match Messages::find()
        .join(JoinType::InnerJoin, messages::Relation::Channels.def())
        .filter(channels::Column::OrganizationId.eq(auth.organization_id))
        .filter(messages::Column::CreatedAt.gte(start_of_month))
        .filter(messages::Column::CreatedAt.lt(end_of_month))
        .count(db)
        .await
    {
        Ok(count) => { 
            let response = CountMessageResponse {
                count: count as u32,
                month: now.month() as u8,
            };
            ServerResponse::ok(response)
        },
        Err(err) => ServerResponse::server_error(err, "Failed to fetch messages count"),
    }
}

use crate::entities::{participant, prelude::*};
use crate::utils::ServerResponse;
use axum::{extract::State, response::IntoResponse, Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CreateParticipantResponse {
    id: Uuid,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateParticipantRequest {
    name: String,
}

pub async fn create_participant(
    State(state): State<AppState>,
    Json(payload): Json<CreateParticipantRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_participant");

    let db = &state.db.connection;

    let name = payload.name;

    let new_participant = participant::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name.clone()),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
    };

    match Participant::insert(new_participant).exec(db).await {
        Ok(participant) => {
            let response = CreateParticipantResponse {
                id: participant.last_insert_id,
                name,
            };
            ServerResponse::created(response)
        }
        Err(err) => ServerResponse::server_error(err, "Failed to create participant"),
    }
}

pub async fn get_participants_count(State(state): State<AppState>) -> impl IntoResponse {
    let db = &state.db.connection;

    match Participant::find().count(db).await {
        Ok(count) => ServerResponse::ok(count),
        Err(err) => ServerResponse::server_error(err, "Failed to get participants count"),
    }
}

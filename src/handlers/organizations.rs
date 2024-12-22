use crate::entities::{api_keys, organization_members};
use crate::entities::prelude::{ApiKeys, OrganizationMembers, Users};
use crate::entities::sea_orm_active_enums::ApiKeyType;
use crate::middleware::{ApiKeyManager, AuthorizedOrganizationUser};
use crate::state::AppState;
use crate::utils::{hash_password_and_salt, ServerResponse};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    name: String,
    key_type: ApiKeyType,
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    id: Uuid,
    name: String,
    key: String,
    key_type: ApiKeyType,
    created_at: chrono::DateTime<Utc>,
}

pub async fn get_api_keys(
    State(state): State<AppState>,
    auth: AuthorizedOrganizationUser,
) -> impl IntoResponse {
    let db = &state.db.connection;

    match ApiKeys::find()
        .filter(api_keys::Column::OrganizationId.eq(auth.organization_id))
        .order_by_asc(api_keys::Column::CreatedAt)
        .all(db)
        .await
    {
        Ok(count) => ServerResponse::ok(count),
        Err(err) => ServerResponse::server_error(err, "Failed to get keys"),
    }
}

pub async fn get_api_key_count(
    State(state): State<AppState>,
    auth: AuthorizedOrganizationUser,
) -> impl IntoResponse {
    let db = &state.db.connection;

    match ApiKeys::find()
        .filter(api_keys::Column::OrganizationId.eq(auth.organization_id))
        .count(db)
        .await
    {
        Ok(count) => ServerResponse::ok(count),
        Err(err) => ServerResponse::server_error(err, "Failed to get keys count"),
    }
}

pub async fn create_api_key(
    State(state): State<AppState>,
    auth: ApiKeyManager,
    Json(payload): Json<CreateApiKeyRequest>,
) -> impl IntoResponse {
    let db = &state.db.connection;
    let now = Utc::now().naive_utc();
    let auth_user = auth.0;
    
    // Generate a unique API key
    let api_key = format!("sk_{}", Uuid::new_v4());
    let hashed_api_key = hash_password_and_salt(&api_key).unwrap();

    // Create new API key
    let new_api_key = api_keys::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(auth_user.organization_id),
        name: Set(payload.name),
        key: Set(hashed_api_key),
        key_type: Set(payload.key_type),
        created_by_user_id: Set(auth_user.user.user_id),
        last_used_at: Set(None),
        expires_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };

    match new_api_key.insert(db).await {
        Ok(api_key_model) => {
            let response = CreateApiKeyResponse {
                id: api_key_model.id,
                name: api_key_model.name,
                key: api_key,
                key_type: api_key_model.key_type,
                created_at: api_key_model.created_at.and_utc(),
            };
            ServerResponse::created(response)
        }
        Err(err) => ServerResponse::server_error(err, "Failed to create API key"),
    }
}


pub async fn get_users_in_org(
    State(state): State<AppState>,
    auth: AuthorizedOrganizationUser,
) -> impl IntoResponse {
    let db = &state.db.connection;

    match Users::find()
        .inner_join(OrganizationMembers)
        .filter(organization_members::Column::OrganizationId.eq(auth.organization_id))
        .all(db)
        .await
    {
        Ok(users) => ServerResponse::ok(users),
        Err(err) => ServerResponse::server_error(err, "Failed to fetch organization users"),
    }
}

pub async fn get_users_in_org_count(
    State(state): State<AppState>,
    auth: AuthorizedOrganizationUser,
) -> impl IntoResponse {
    tracing::info!("executes: get_users_in_organization_count");

    let db = &state.db.connection;
    match Users::find()
        .inner_join(OrganizationMembers)
        .filter(organization_members::Column::OrganizationId.eq(auth.organization_id))
        .count(db)
        .await
    {
        Ok(users) => ServerResponse::ok(users),
        Err(err) => ServerResponse::server_error(err, "Failed to fetch org users count"),
    }

}

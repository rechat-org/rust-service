use crate::entities::prelude::{ApiKeys, OrganizationMembers, Users};
use crate::entities::sea_orm_active_enums::{ApiKeyType, OrganizationRole};
use crate::entities::{api_keys, organization_members, users};
use crate::state::AppState;
use crate::utils::{generate_api_key_prefix, hash_password_and_salt, ServerResponse};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use sea_orm::*;
use sea_orm::prelude::DateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::middleware::authorization::{ApiKeyManager, AuthorizedOrganizationUser};

#[derive(Debug, Serialize, FromQueryResult)]
pub struct OrgUserResponse {
    id: Uuid,
    email: String,
    role: OrganizationRole,
    created_at: DateTime,
}

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
#[derive(Debug, Serialize)]
pub struct GetApiKeysResponse {
    id: Uuid,
    name: String,
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
        Ok(api_keys) => ServerResponse::ok({
            api_keys
                .into_iter()
                .map(|key| GetApiKeysResponse {
                    id: key.id,
                    name: key.name,
                    key_type: key.key_type,
                    created_at: key.created_at.and_utc(),
                })
                .collect::<Vec<GetApiKeysResponse>>()
        }),
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

    // Generate the key prefix
    let key_prefix = generate_api_key_prefix(&api_key);

    // Hash the API key
    let hashed_api_key = match hash_password_and_salt(&api_key) {
        Ok(hashed) => hashed,
        Err(err) => return ServerResponse::server_error(err, "Failed to hash API key"),
    };

    // Create new API key
    let new_api_key = api_keys::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(auth_user.organization_id),
        name: Set(payload.name),
        key: Set(hashed_api_key),
        key_prefix: Set(key_prefix), 
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
                key: api_key,  // Uppon creation we want to return the unhashed key to the user
                key_type: api_key_model.key_type,
                created_at: api_key_model.created_at.and_utc(),
            };
            ServerResponse::created(response)
        }
        Err(err) => ServerResponse::server_error(err, "Failed to create API key"),
    }
}

pub async fn delete_api_key(
    State(state): State<AppState>,
    Path((organization_id, key_id)): Path<(Uuid, Uuid)>, // Extract both IDs
    auth: ApiKeyManager,
) -> impl IntoResponse {
    let db = &state.db.connection;
    let auth_user = auth.0;

    // Verifying organization_id matches the authenticated user's org
    if organization_id != auth_user.organization_id {
        return ServerResponse::forbidden("Not authorized to access this organization");
    }

    // First checking if the key exists and belongs to the organization
    let key = match ApiKeys::find_by_id(key_id)
        .filter(api_keys::Column::OrganizationId.eq(organization_id))
        .one(db)
        .await
    {
        Ok(Some(key)) => key,
        Ok(None) => return ServerResponse::not_found("API key not found"),
        Err(err) => return ServerResponse::server_error(err, "Failed to find API key"),
    };

    // Delete the key
    match ApiKeys::delete_by_id(key.id).exec(db).await {
        Ok(_) => ServerResponse::ok(()),
        Err(err) => ServerResponse::server_error(err, "Failed to delete API key"),
    }
}

pub async fn get_users_in_org(
    State(state): State<AppState>,
    auth: AuthorizedOrganizationUser,
) -> impl IntoResponse {
    let db = &state.db.connection;

    let users = Users::find()
        .select_only()
        .column(users::Column::Id)
        .column(users::Column::Email)
        .column(users::Column::CreatedAt)
        .column(organization_members::Column::Role)
        .join(
            JoinType::InnerJoin,
            users::Relation::OrganizationMembers.def()
        )
        .filter(
            organization_members::Column::OrganizationId.eq(auth.organization_id)
        )
        .into_model::<OrgUserResponse>()
        .all(db)
        .await;

    match users {
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

pub async fn get_active_users(
    State(state): State<AppState>,
    _: AuthorizedOrganizationUser, // Use auth middleware
) -> impl IntoResponse {
    let count = *state.active_users.read().await;
    ServerResponse::ok(count)
}
use crate::entities::organization_tiers;
use crate::utils::generate_api_key_prefix;
use crate::{
    entities::{api_keys, prelude::*},
    middleware::error::MiddlewareError,
    state::AppState,
};
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use bcrypt::verify;
use chrono::Utc;
use sea_orm::prelude::Expr;
use sea_orm::*;
use tokio::spawn;
use uuid::Uuid;

pub(crate) async fn find_and_validate_key(
    api_key: &str,
    organization_id: &Uuid,
    db: &DatabaseConnection,
) -> Result<api_keys::Model, MiddlewareError> {
    let key_prefix = generate_api_key_prefix(api_key);

    let potential_keys = api_keys::Entity::find()
        .filter(api_keys::Column::KeyPrefix.eq(key_prefix))
        .filter(api_keys::Column::OrganizationId.eq(*organization_id))
        .all(db)
        .await
        .map_err(|e| MiddlewareError::DatabaseError(e.to_string()))?;

    let key = potential_keys
        .into_iter()
        .find(|k| verify(api_key, &k.key).unwrap_or(false))
        .ok_or_else(|| MiddlewareError::InvalidToken("Invalid API key".into()))?;

    // Checks expiration first
    if let Some(expires_at) = key.expires_at {
        if expires_at < Utc::now().naive_utc() {
            return Err(MiddlewareError::ExpiredToken);
        }
    }

    // We clone what we need for the background task
    let db = db.clone();
    let key_id = key.id;

    // Update last_used_at in the background so that we don't block the response
    spawn(async move {
        let now = Utc::now().naive_utc();

        let key_active: api_keys::ActiveModel = api_keys::ActiveModel {
            id: Set(key_id),
            last_used_at: Set(Some(now)),
            updated_at: Set(now),
            ..Default::default()
        };

        if let Err(e) = key_active.update(&db).await {
            tracing::error!("Failed to update key last_used_at: {}", e);
        }
    });

    Ok(key)
}

// Helper function to extract API key from headers
pub(crate) fn extract_api_key(parts: &Parts) -> Result<String, MiddlewareError> {
    parts
        .headers
        .get("X-API-Key")
        .ok_or(MiddlewareError::MissingToken)?
        .to_str()
        .map(String::from)
        .map_err(|_| MiddlewareError::InvalidToken("Invalid header value".into()))
}

pub(crate) async fn extract_organization_id(
    parts: &mut Parts,
    state: &AppState,
) -> Result<Uuid, MiddlewareError> {
    // Try standard organization path first
    if let Ok(Path(org_path)) = Path::<(Uuid,)>::from_request_parts(parts, state).await {
        return Ok(org_path.0);
    }

    // Try nested organization path next
    if let Ok(Path(nested_path)) = Path::<(Uuid, Uuid)>::from_request_parts(parts, state).await {
        return Ok(nested_path.0);
    }

    Err(MiddlewareError::OrgNotFound)
}

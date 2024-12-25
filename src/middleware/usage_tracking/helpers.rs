use crate::{
    entities::{api_keys, prelude::*},
    middleware::error::AuthError,
    state::AppState,
};
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use bcrypt::verify;
use sea_orm::*;
use uuid::Uuid;

pub(crate) async fn find_and_validate_key(
    api_key: &str,
    organization_id: &Uuid,
    db: &DatabaseConnection,
) -> Result<api_keys::Model, AuthError> {
    // Get all api keys since we can't compare hashed values directly in DB
    let keys = ApiKeys::find()
        .all(db)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Find the key that matches when verified with bcrypt
    let key = keys
        .into_iter()
        .find(|k| k.organization_id == *organization_id && verify(api_key, &k.key).unwrap_or(false))
        .ok_or_else(|| AuthError::InvalidToken("Invalid API key".into()))?;

    // Check expiration
    if let Some(expires_at) = key.expires_at {
        if expires_at < chrono::Utc::now().naive_utc() {
            return Err(AuthError::ExpiredToken);
        }
    }

    Ok(key)
}

pub(crate) async fn track_api_usage(state: &AppState, org_id: &Uuid) -> Result<(), AuthError> {
    let org = match Organizations::find_by_id(*org_id)
        .one(&state.db.connection)
        .await
    {
        Ok(Some(org)) => org,
        Ok(None) => return Err(AuthError::DatabaseError("Organization not found".into())),
        Err(e) => return Err(AuthError::DatabaseError(e.to_string())),
    };

    // If no stripe customer id, early return success
    let Some(stripe_customer_id) = org.stripe_customer_id else {
        return Ok(());
    };

    println!(
        "Reporting usage to Stripe for stripe_customer_id {}",
        stripe_customer_id
    );
    // Report usage to Stripe, but don't fail if reporting fails
    if let Err(e) = state.stripe.report_api_usage(&stripe_customer_id).await {
        tracing::error!("Failed to report usage to Stripe: {}", e);
    }

    Ok(())
}

// Helper function to extract API key from headers
pub(crate) fn extract_api_key(parts: &Parts) -> Result<String, AuthError> {
    parts
        .headers
        .get("X-API-Key")
        .ok_or(AuthError::MissingToken)?
        .to_str()
        .map(String::from)
        .map_err(|_| AuthError::InvalidToken("Invalid header value".into()))
}

pub(crate) async fn extract_organization_id(
    parts: &mut Parts,
    state: &AppState,
) -> Result<Uuid, AuthError> {
    // Try standard organization path first
    if let Ok(Path(org_path)) = Path::<(Uuid,)>::from_request_parts(parts, state).await {
        return Ok(org_path.0);
    }

    // Try nested organization path next
    if let Ok(Path(nested_path)) = Path::<(Uuid, Uuid)>::from_request_parts(parts, state).await {
        return Ok(nested_path.0);
    }

    Err(AuthError::OrgNotFound)
}

use crate::{
    entities::{api_keys, prelude::*},
    middleware::error::AuthError,
    state::AppState,
};
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use bcrypt::verify;
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;
use crate::utils::generate_api_key_prefix;

pub(crate) async fn find_and_validate_key(
    api_key: &str,
    organization_id: &Uuid,
    db: &DatabaseConnection,
) -> Result<api_keys::Model, AuthError> {
    // Generate prefix for the incoming key
    let key_prefix = generate_api_key_prefix(api_key);

    // Find potential matches using the prefix and org_id
    let potential_keys = api_keys::Entity::find()
        .filter(api_keys::Column::KeyPrefix.eq(key_prefix))
        .filter(api_keys::Column::OrganizationId.eq(*organization_id))
        .all(db)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Find the key that matches when verified with bcrypt
    let key = potential_keys
        .into_iter()
        .find(|k| verify(api_key, &k.key).unwrap_or(false))
        .ok_or_else(|| AuthError::InvalidToken("Invalid API key".into()))?;

    // Update last_used_at timestamp
    let mut key_active: api_keys::ActiveModel = key.clone().into();
    key_active.last_used_at = Set(Some(Utc::now().naive_utc()));
    key_active.updated_at = Set(Utc::now().naive_utc());

    let updated_key = key_active
        .update(db)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Check expiration
    if let Some(expires_at) = updated_key.expires_at {
        if expires_at < Utc::now().naive_utc() {
            return Err(AuthError::ExpiredToken);
        }
    }

    Ok(updated_key)
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
    let Some(stripe_subscription_id) = org.stripe_subscription_id else {
        return Ok(());
    };

    println!(
        "Reporting usage to Stripe for stripe_customer_id {}",
        stripe_subscription_id
    );
    // Report usage to Stripe, but don't fail if reporting fails
    if let Err(e) = state.stripe.report_api_usage(&stripe_subscription_id).await {
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

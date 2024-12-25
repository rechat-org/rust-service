use crate::entities::sea_orm_active_enums::ApiKeyType;
use crate::middleware::usage_tracking::helpers::{
    extract_api_key, extract_organization_id, find_and_validate_key, track_api_usage,
};
use crate::{middleware::error::AuthError, state::AppState};
use axum::extract::Path;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;
#[derive(Debug, Clone)]
pub struct ApiKeyAuthorizer {
    pub key_type: ApiKeyType,
}

#[async_trait]
impl FromRequestParts<AppState> for ApiKeyAuthorizer {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let org_id = extract_organization_id(parts, state).await?;
        let api_key = extract_api_key(parts)?;
        let key = find_and_validate_key(&api_key, &org_id, &state.db.connection).await?;

        Ok(Self {
            key_type: key.key_type,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UsageTracker {}

#[async_trait]
impl FromRequestParts<AppState> for UsageTracker {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path((org_id, _)): Path<(Uuid, Uuid)> = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::MissingOrgId)?;

        if let Err(e) = track_api_usage(state, &org_id).await {
            tracing::error!("Usage tracking failed: {}", e);
        }

        Ok(Self {})
    }
}

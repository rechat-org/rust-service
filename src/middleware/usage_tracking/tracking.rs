use crate::entities::sea_orm_active_enums::ApiKeyType;
use crate::middleware::usage_tracking::helpers::{
    extract_api_key, extract_organization_id_from_path, find_and_validate_key, track_api_usage,
};
use crate::{middleware::error::AuthError, state::AppState};
use axum::extract::Path;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub api_key: String,
    pub organization_id: Uuid,
    pub key_type: ApiKeyType,
}

#[async_trait]
impl FromRequestParts<AppState> for ApiKeyAuth {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let api_key = extract_api_key(parts)?;
        let organization_id = extract_organization_id_from_path(parts, state).await?;
        
        let key = find_and_validate_key(&api_key, &organization_id, &state.db.connection).await?;

        Ok(Self {
            api_key,
            organization_id: key.organization_id,
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
        let organization_id = extract_organization_id_from_path(parts, state).await?;

        if let Err(e) = track_api_usage(state, &organization_id).await {
            tracing::error!("Usage tracking failed: {}", e);
        }

        Ok(Self {})
    }
}

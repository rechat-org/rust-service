use crate::middleware::helpers::{extract_organization_id, track_api_usage};
use crate::{middleware::error::AuthError, state::AppState};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

#[derive(Debug, Clone)]
pub struct UsageTracker {}

#[async_trait]
impl FromRequestParts<AppState> for UsageTracker {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let org_id = extract_organization_id(parts, state).await?;

        if let Err(e) = track_api_usage(state, &org_id).await {
            tracing::error!("Usage tracking failed: {}", e);
        }

        Ok(Self {})
    }
}

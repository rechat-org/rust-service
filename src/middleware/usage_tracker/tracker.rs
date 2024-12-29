use crate::middleware::helpers::{extract_organization_id, track_api_usage};
use crate::{middleware::error::AppError, state::AppState};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use tokio::spawn;

#[derive(Debug, Clone)]
pub struct UsageTracker {}

#[async_trait]
impl FromRequestParts<AppState> for UsageTracker {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let org_id = extract_organization_id(parts, state).await?;

        // We clone what we need for the background task
        let state = state.clone();
        let org_id = org_id.clone();

        // Spawn because we want the tracking to be happening in the bg background only
        spawn(async move {
            if let Err(e) = track_api_usage(&state, &org_id).await {
                tracing::error!("Background usage tracking failed: {}", e);
            }
        });

        Ok(Self {})
    }
}

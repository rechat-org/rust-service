use crate::entities::prelude::{OrganizationTiers, Organizations};
use crate::middleware::error::MiddlewareError;
use crate::middleware::helpers::extract_organization_id;
use crate::state::AppState;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use sea_orm::*;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UsageLimiter;

#[async_trait]
impl FromRequestParts<AppState> for UsageLimiter {
    type Rejection = MiddlewareError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract org_id, log error to devs but return generic error to user
        let org_id = match extract_organization_id(parts, state).await {
            Ok(id) => id,
            Err(e) => {
                tracing::error!("Failed to extract organization ID: {}", e);
                return Ok(Self); // Let the request through rather than error
            }
        };

        // Check usage, log error to devs but return generic success to user
        let usage = match check_usage(&state, &org_id).await {
            Ok(u) => u,
            Err(e) => {
                tracing::error!("Failed to check usage for org {}: {}", org_id, e);
                return Ok(Self); // Let the request through if we can't check usage
            }
        };

        // Get tier info, log error to devs but return generic success to user
        let org_tier = match OrganizationTiers::find_by_id(org_id)
            .one(&state.db.connection)
            .await
        {
            Ok(Some(tier)) => tier,
            Ok(None) => {
                tracing::error!("Organization tier not found for org: {}", org_id);
                return Ok(Self); // Let the request through if we can't find tier
            }
            Err(e) => {
                tracing::error!("Database error getting org tier: {}", e);
                return Ok(Self); // Let the request through on DB errors
            }
        };

        let tier_limit = org_tier.monthly_request_limit;

        // This is the only error we want to show to users
        if usage >= tier_limit {
            let error_message = format!(
                "Usage limit exceeded. Current usage: {}, Tier limit: {}. Please upgrade your subscription.",
                usage,
                tier_limit
            );
            return Err(MiddlewareError::UsageLimitExceeded(error_message));
        }

        // Threshold notification in background
        let threshold_percentage = 0.8;
        let threshold = (tier_limit as f64 * threshold_percentage) as i64;

        if usage > threshold {
            let state = state.clone();
            let org_id = org_id.clone();
            tokio::spawn(async move {
                if let Err(e) = notify_usage_threshold(&state, &org_id, usage, tier_limit).await {
                    tracing::error!("Failed to send usage notification: {}", e);
                    // Error is only logged, not returned to user
                }
            });
        }

        Ok(Self)
    }
}

async fn check_usage(state: &AppState, org_id: &Uuid) -> Result<i64, MiddlewareError> {
    // Try Redis first
    match state.redis.get_usage(org_id).await {
        Ok(Some(usage)) => return Ok(usage),
        Ok(None) => {
            tracing::debug!("No usage found in Redis for org: {}", org_id);
            // Continue to Stripe check
        }
        Err(e) => {
            tracing::error!("Redis error checking usage: {}", e);
            // Continue to Stripe check
        }
    }

    // Fallback to Stripe
    let usage = state
        .stripe
        .get_subscription_usage(&state.db.connection, org_id)
        .await;

    // Try to cache the result, but only log errors
    if let Err(e) = state.redis.set_usage(org_id, usage).await {
        tracing::error!("Failed to cache usage in Redis: {}", e);
    }

    Ok(usage)
}

async fn notify_usage_threshold(
    state: &AppState,
    org_id: &Uuid,
    current_usage: i64,
    tier_limit: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get org details, log errors but don't propagate
    let org_name = match Organizations::find_by_id(*org_id)
        .one(&state.db.connection)
        .await
    {
        Ok(Some(org)) => org.name,
        Ok(None) => {
            tracing::error!("Organization not found for notification: {}", org_id);
            return Ok(()); // Silent failure
        }
        Err(e) => {
            tracing::error!("Database error in notification: {}", e);
            return Ok(()); // Silent failure
        }
    };

    let percentage_used = (current_usage as f64 / tier_limit as f64 * 100.0) as i32;

    // Log for devs
    tracing::warn!(
        "Organization {} has used {}% of their monthly limit ({}/{})",
        org_name,
        percentage_used,
        current_usage,
        tier_limit
    );

    // TODO: Implement actual notification
    // Any errors in notification sending should only be logged, not propagated

    Ok(())
}
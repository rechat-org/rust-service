use crate::config::RedisStore;
use crate::entities::prelude::Organizations;
use crate::middleware::error::AuthError;
use crate::middleware::helpers::extract_organization_id;
use crate::state::AppState;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use chrono::Timelike;
use redis::AsyncCommands;
use sea_orm::*;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UsageLimiter;

#[async_trait]
impl FromRequestParts<AppState> for UsageLimiter {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let org_id = extract_organization_id(parts, state).await?;

        // Checks Redis cache first for usage
        let usage = check_and_increment_usage(&state.redis, &org_id).await?;

        // Gets subscription info from Stripe
        let subscription = state
            .stripe
            .get_subscription(&org_id, &state.db.connection)
            .await
            .map_err(|e| AuthError::StripeError(e.to_string()))?;

        // Extracts tier limit from subscription metadata
        let tier_limit: i64 = subscription
            .metadata
            .get("monthly_limit")
            .and_then(|l| l.parse().ok())
            .ok_or_else(|| AuthError::ConfigError("Invalid tier limit".into()))?;

        // Checks if org is within limits
        if usage > tier_limit {
            let error_message = format!(
                "Usage limit exceeded. Current usage: {}, Tier limit: {}. Please upgrade your subscription.",
                usage,
                tier_limit
            );
            return Err(AuthError::UsageLimitExceeded(error_message));
        }

        // If approaching limit (80%), spawn background notification task
        let threshold_percentage = 0.8;
        let tier_limit_float = tier_limit as f64;
        let threshold = (tier_limit_float * threshold_percentage) as i64;

        if usage > threshold {
            let state = state.clone();
            let org_id = org_id.clone();
            tokio::spawn(async move {
                if let Err(e) = notify_usage_threshold(&state, &org_id, usage, tier_limit).await {
                    tracing::error!("Failed to send usage notification: {}", e);
                }
            });
        }

        Ok(Self)
    }
}

// Helper functions
async fn check_and_increment_usage(redis: &RedisStore, org_id: &Uuid) -> Result<i64, AuthError> {
    let mut conn = redis
        .client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AuthError::CacheError(e.to_string()))?;

    let key = format!(
        "usage:monthly:{}:{}",
        chrono::Utc::now().format("%Y-%m"),
        org_id
    );

    // Increment usage counter
    let new_count: i64 = conn
        .incr(&key, 1)
        .await
        .map_err(|e| AuthError::CacheError(e.to_string()))?;

    // Set expiry if this is a new key
    if new_count == 1 {
        let seconds_until_month_end = calculate_seconds_until_month_end();
        let _: () = conn
            .expire(&key, seconds_until_month_end)
            .await
            .map_err(|e| AuthError::CacheError(e.to_string()))?;
    }

    Ok(new_count)
}

fn calculate_seconds_until_month_end() -> i64 {
    use chrono::{Datelike, Duration, Utc};

    let now = Utc::now();
    let next_month = if now.month() == 12 {
        // For December, go to next year January
        Utc::now()
            .with_year(now.year() + 1)
            .unwrap()
            .with_month(1)
            .unwrap()
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
    } else {
        // For other months, just go to next month
        Utc::now()
            .with_month(now.month() + 1)
            .unwrap()
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
    };

    (next_month - now).num_seconds()
}
async fn notify_usage_threshold(
    state: &AppState,
    org_id: &Uuid,
    current_usage: i64,
    tier_limit: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get organization details
    let org = Organizations::find_by_id(*org_id)
        .one(&state.db.connection)
        .await?
        .ok_or("Organization not found")?;

    // Calculate percentage used
    let percentage_used = (current_usage as f64 / tier_limit as f64 * 100.0) as i32;

    // Send notification (implement based on your notification system)
    // This could be email, Slack, etc.
    tracing::warn!(
        "Organization {} has used {}% of their monthly limit ({}/{})",
        org.name,
        percentage_used,
        current_usage,
        tier_limit
    );

    Ok(())
}

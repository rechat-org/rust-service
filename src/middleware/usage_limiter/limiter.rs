use crate::config::RedisStore;
use crate::entities::prelude::{OrganizationTiers, Organizations};
use crate::middleware::error::MiddlewareError;
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
    type Rejection = MiddlewareError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let org_id = extract_organization_id(parts, state).await?;

        // Checks Redis cache first for usage
        let usage = check_and_increment_usage(&state, &org_id).await?;

        // Get organization tier info from database
        let org_tier = OrganizationTiers::find_by_id(org_id)
            .one(&state.db.connection)
            .await
            .map_err(|e| MiddlewareError::DatabaseError(e.to_string()))?
            .ok_or_else(|| MiddlewareError::NotFound("Organization tier not found".into()))?;

        // Get monthly limit from database
        let tier_limit = org_tier.monthly_request_limit;

        // Checks if org is within limits
        if usage > tier_limit {
            let error_message = format!(
                "Usage limit exceeded. Current usage: {}, Tier limit: {}. Please upgrade your subscription.",
                usage,
                tier_limit
            );
            return Err(MiddlewareError::UsageLimitExceeded(error_message));
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

async fn check_and_increment_usage(
    state: &AppState,
    org_id: &Uuid,
) -> Result<i64, MiddlewareError> {
    let key = format!(
        "usage:monthly:{}:{}",
        chrono::Utc::now().format("%Y-%m"),
        org_id
    );

    // Try Redis first
    match state.redis.client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            // First try to get the existing value
            let current_count: Option<i64> = conn
                .get(&key)
                .await
                .map_err(|e| MiddlewareError::CacheError(e.to_string()))?;

            match current_count {
                Some(count) => {
                    // Key exists, increment it
                    let new_count: i64 = conn
                        .incr(&key, 1)
                        .await
                        .map_err(|e| MiddlewareError::CacheError(e.to_string()))?;
                    Ok(new_count)
                }
                None => {
                    // Key doesn't exist, get current usage from DB
                    let org_tier = OrganizationTiers::find_by_id(*org_id)
                        .one(&state.db.connection)
                        .await
                        .map_err(|e| MiddlewareError::DatabaseError(e.to_string()))?
                        .ok_or_else(|| {
                            MiddlewareError::NotFound("Organization tier not found".into())
                        })?;

                    let current_usage = org_tier.current_month_usage + 1;

                    // Set in Redis with expiry
                    let _: () = conn
                        .set_ex(&key, current_usage, calculate_seconds_until_month_end())
                        .await
                        .map_err(|e| MiddlewareError::CacheError(e.to_string()))?;

                    Ok(current_usage)
                }
            }
        }
        Err(redis_err) => {
            tracing::warn!("Redis error, starting fresh counter: {}", redis_err);
            // If Redis fails, just return DB value + 1
            let org_tier = OrganizationTiers::find_by_id(*org_id)
                .one(&state.db.connection)
                .await
                .map_err(|e| MiddlewareError::DatabaseError(e.to_string()))?
                .ok_or_else(|| MiddlewareError::NotFound("Organization tier not found".into()))?;

            Ok(org_tier.current_month_usage + 1)
        }
    }
}

fn calculate_seconds_until_month_end() -> u64 {
    use chrono::{Datelike, Utc};

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

    (next_month - now).num_seconds() as u64
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

    // TODO: We want to send a notification to the organization owner
    tracing::warn!(
        "Organization {} has used {}% of their monthly limit ({}/{})",
        org.name,
        percentage_used,
        current_usage,
        tier_limit
    );

    Ok(())
}

use crate::entities::organization_tiers;
use crate::entities::prelude::{OrganizationTiers, Organizations};
use crate::middleware::helpers::extract_organization_id;
use crate::{middleware::error::MiddlewareError, state::AppState};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use sea_orm::prelude::Expr;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use tokio::spawn;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UsageTracker {}

#[async_trait]
impl FromRequestParts<AppState> for UsageTracker {
    type Rejection = MiddlewareError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let org_id = extract_organization_id(parts, state).await?;

        // We clone what we need for the background task
        let state = state.clone();
        let org_id = org_id.clone();

        // Spawn the tracking/incrementing task in the background
        spawn(async move {
            if let Err(e) = track_api_usage(&state, &org_id).await {
                tracing::error!("Background usage tracking failed: {}", e);
            }
        });

        Ok(Self {})
    }
}

pub(crate) async fn track_api_usage(
    state: &AppState,
    org_id: &Uuid,
) -> Result<(), MiddlewareError> {
    // First, increment Redis synchronously as it's our primary source for usage checks
    match state.redis.increment_usage(org_id).await {
        Ok(_) => {
            tracing::debug!(
                "Successfully incremented usage in Redis for org: {}",
                org_id
            );
        }
        Err(e) => {
            tracing::error!("Failed to increment Redis usage counter: {}", e);
            // Don't return here, continue with other operations
        }
    }

    // Clone state and org_id for background operations
    let state = state.clone();
    let org_id = *org_id;

    // Spawn background task for Stripe and DB updates
    spawn(async move {
        // Get organization details
        let org = match Organizations::find_by_id(org_id)
            .one(&state.db.connection)
            .await
        {
            Ok(Some(org)) => org,
            Ok(None) => {
                tracing::error!("Organization not found for usage tracking: {}", org_id);
                return;
            }
            Err(e) => {
                tracing::error!("Database error in usage tracking: {}", e);
                return;
            }
        };

        // Early return if no stripe subscription
        let Some(stripe_subscription_item_id) = org.stripe_subscription_item_id else {
            return;
        };

        // Spawn Stripe update
        let stripe_client = state.stripe.clone();
        let stripe_sub_id = stripe_subscription_item_id.clone();
        spawn(async move {
            if let Err(e) = stripe_client.report_api_usage(&stripe_sub_id).await {
                tracing::error!("Failed to report usage to Stripe: {}", e);
            }
        });

        // Spawn database update
        let db = state.db.connection.clone();
        let db_org_id = org_id;
        spawn(async move {
            let update_result = OrganizationTiers::update_many()
                .col_expr(
                    organization_tiers::Column::CurrentMonthUsage,
                    Expr::col(organization_tiers::Column::CurrentMonthUsage).add(1),
                )
                .filter(organization_tiers::Column::OrganizationId.eq(db_org_id))
                .exec(&db)
                .await;

            if let Err(e) = update_result {
                tracing::error!("Failed to update organization tier usage: {}", e);
            }
        });

        // If Redis increment failed earlier, try to get current usage from Stripe and set it
        if state.redis.get_usage(&org_id).await.is_err() {
            let current_usage = state
                .stripe
                .get_subscription_usage(&state.db.connection, &org_id)
                .await;

            if let Err(e) = state.redis.set_usage(&org_id, current_usage).await {
                tracing::error!(
                    "Failed to set usage in Redis after increment failure: {}",
                    e
                );
            }
        }
    });

    Ok(())
}

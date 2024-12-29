use crate::entities::prelude::{OrganizationTiers, Organizations};
use crate::utils::GeneralError;
use chrono::Utc;
use reqwest::Client as HttpClient;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
use std::str::FromStr;
use stripe::{
    Client, CreateCustomer, CreateSubscription, CreateSubscriptionItems, Customer, StripeError,
    Subscription, SubscriptionId,
};
use thiserror::Error;
use uuid::Uuid;

// Custom error type for Stripe operations
#[derive(Debug, Error)]
pub enum StripeClientError {
    #[error("Stripe error: {0}")]
    Stripe(#[from] StripeError),
    #[error("Configuration error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct StripeClient {
    pub(crate) client: Client,
}

pub struct SubscriptionInfo {
    pub id: String,
    pub monthly_limit: i64,
    pub status: stripe::SubscriptionStatus,
    pub metadata: std::collections::HashMap<String, String>,
}

impl StripeClient {
    pub fn new() -> Result<Self, DbErr> {
        let secret_key =
            std::env::var("STRIPE_SECRET_KEY").expect("Missing STRIPE_SECRET_KEY in env");

        Ok(Self {
            client: Client::new(secret_key.clone()),
        })
    }

    /// Creates a customer and subscription (unchanged from your snippet)
    pub async fn create_customer_with_subscription(
        &self,
        email: &str,
        organization_name: &str,
    ) -> Result<(String, String, String), StripeError> {
        // Create customer
        let customer = Customer::create(
            &self.client,
            CreateCustomer {
                name: Some(organization_name),
                email: Some(email),
                metadata: Some(std::collections::HashMap::from([(
                    String::from("async-stripe"),
                    String::from("true"),
                )])),
                ..Default::default()
            },
        )
        .await?;

        // Creates subscription for customer
        let price_id = std::env::var("STRIPE_PRICE_ID").expect("Missing STRIPE_PRICE_ID in env");
        let mut params = CreateSubscription::new(customer.id.clone());
        params.items = Some(vec![CreateSubscriptionItems {
            price: Some(price_id),
            ..Default::default()
        }]);
        params.expand = &["items", "items.data.price.product", "schedule"];

        let subscription = Subscription::create(&self.client, params).await?;
        let subscription_item_id = subscription.items.data[0].id.clone();

        Ok((
            customer.id.to_string(),
            subscription.id.to_string(),
            subscription_item_id.to_string(),
        ))
    }

    pub async fn get_subscription(
        &self,
        org_id: &Uuid,
        db: &DatabaseConnection,
    ) -> Result<SubscriptionInfo, GeneralError> {
        // Get organization from database
        let org = Organizations::find_by_id(*org_id)
            .one(db)
            .await
            .map_err(|e| GeneralError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| GeneralError::NotFound(format!("Organization {} not found", org_id)))?;

        // Check if org has a subscription
        let subscription_id = org.stripe_subscription_id.ok_or_else(|| {
            GeneralError::BadRequest(format!(
                "Organization {} has no active subscription",
                org_id
            ))
        })?;

        // Get subscription from Stripe
        let subscription = match Subscription::retrieve(
            &self.client,
            &SubscriptionId::from_str(&subscription_id).map_err(|_| {
                GeneralError::Internal("Invalid subscription ID format".to_string())
            })?,
            &["items.data.price.product"],
        )
        .await
        {
            Ok(sub) => sub,
            Err(e) => {
                tracing::error!("Stripe API error: {:?}", e);
                return Err(GeneralError::Internal(
                    "Failed to fetch subscription from Stripe".to_string(),
                ));
            }
        };

        // Get monthly limit from organization_tiers table
        let org_tier = OrganizationTiers::find_by_id(*org_id)
            .one(db)
            .await
            .map_err(|e| GeneralError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| {
                GeneralError::NotFound(format!("Organization tier {} not found", org_id))
            })?;

        Ok(SubscriptionInfo {
            id: subscription.id.to_string(),
            monthly_limit: org_tier.monthly_request_limit,
            status: subscription.status,
            metadata: subscription.metadata,
        })
    }

    pub async fn report_api_usage(
        &self,
        stripe_subscription_item_id: &str,
    ) -> Result<(), StripeClientError> {
        tracing::info!(
            "Reporting usage to Stripe for stripe_subscription_item_id {}",
            stripe_subscription_item_id
        );
        // Handle env var error gracefully
        let secret_key = match std::env::var("STRIPE_SECRET_KEY") {
            Ok(key) => key,
            Err(e) => {
                tracing::error!("Failed to get STRIPE_SECRET_KEY: {}", e);
                return Ok(());
            }
        };

        let timestamp = Utc::now().timestamp().to_string();

        let params = [
            ("action", "increment"),
            ("quantity", "1"),
            ("timestamp", timestamp.as_str()),
        ];

        let http_client = HttpClient::new();
        let resp = match http_client
            .post(&format!(
                "https://api.stripe.com/v1/subscription_items/{}/usage_records",
                stripe_subscription_item_id
            ))
            .header("Authorization", format!("Bearer {}", secret_key))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("Failed to send request to Stripe: {}", e);
                return Ok(());
            }
        };

        let status = resp.status();
        let response_text = match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("Failed to read response text: {}", e);
                return Ok(());
            }
        };

        if !status.is_success() {
            tracing::error!(
                "Failed to create usage record for subscription item {}, status: {}, body: {}",
                stripe_subscription_item_id,
                status,
                response_text
            );
            return Ok(());
        }

        tracing::info!(
            "Successfully created usage record for subscription item {}",
            stripe_subscription_item_id
        );
        Ok(())
    }

    pub async fn get_subscription_usage(&self, db: &DatabaseConnection, org_id: &Uuid) -> i64 {
        let org = match Organizations::find_by_id(*org_id).one(db).await {
            Ok(Some(org)) => org,
            Ok(None) => {
                tracing::error!("Organization not found: {}", org_id);
                return 0;
            }
            Err(e) => {
                tracing::error!("Database error: {}", e);
                return 0;
            }
        };

        let Some(subscription_item_id) = org.stripe_subscription_item_id else {
            tracing::error!("No subscription item ID found for org: {}", org_id);
            return 0;
        };

        tracing::info!(
            "Fetching usage from Stripe for subscription_item_id {}",
            subscription_item_id
        );

        let secret_key = match std::env::var("STRIPE_SECRET_KEY") {
            Ok(key) => key,
            Err(e) => {
                tracing::error!("Failed to get STRIPE_SECRET_KEY: {}", e);
                return 0;
            }
        };

        let http_client = HttpClient::new();
        let resp = match http_client
            .get(&format!(
                "https://api.stripe.com/v1/subscription_items/{}/usage_record_summaries",
                subscription_item_id
            ))
            .header("Authorization", format!("Bearer {}", secret_key))
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("Failed to send request to Stripe: {}", e);
                return 0;
            }
        };

        let status = resp.status();
        let response_text = match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("Failed to read response text: {}", e);
                return 0;
            }
        };

        if !status.is_success() {
            tracing::error!(
                "Failed to fetch usage records, status: {}, body: {}",
                status,
                response_text
            );
            return 0;
        }

        // Parse the response
        let response = match serde_json::from_str::<serde_json::Value>(&response_text) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to parse Stripe response: {}", e);
                return 0;
            }
        };

        // Get usage value
        response
            .get("data")
            .and_then(|data| data.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("total_usage"))
            .and_then(|usage| usage.as_i64())
            .unwrap_or(0)
    }
}

use chrono::Utc;
use reqwest::Client as HttpClient;
use sea_orm::DbErr;
use serde::Serialize;
use stripe::{
    Client, CreateCustomer, CreateSubscription, CreateSubscriptionItems, CreateUsageRecord,
    Customer, StripeError, Subscription, SubscriptionItemId, UsageRecord, UsageRecordAction,
};
use thiserror::Error;
use uuid::Uuid;

// Custom error type for Stripe operations
#[derive(Debug, Error)]
pub enum StripeClientError {
    #[error("Stripe error: {0}")]
    Stripe(#[from] StripeError),
    #[error("Missing subscription item ID")]
    MissingSubscriptionItem,
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Invalid data error: {0}")]
    InvalidRequestError(String),
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Serialize)]
struct MeterEventPayload {
    stripe_customer_id: String,
}

#[derive(Serialize)]
struct MeterEventRequest {
    identifier: Uuid,
    event_name: String,
    timestamp: String,
    payload: MeterEventPayload,
}
#[derive(Clone)]
pub struct StripeClient {
    pub(crate) client: Client,
    // Add a field for storing the raw secret key if needed
    pub(crate) secret_key: String,
}

impl StripeClient {
    pub fn new() -> Result<Self, DbErr> {
        let secret_key =
            std::env::var("STRIPE_SECRET_KEY").expect("Missing STRIPE_SECRET_KEY in env");

        Ok(Self {
            client: Client::new(secret_key.clone()),
            secret_key,
        })
    }

    /// Creates a customer and subscription (unchanged from your snippet)
    pub async fn create_customer_with_subscription(
        &self,
        email: &str,
        organization_name: &str,
    ) -> Result<(String, String), StripeError> {
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

        // Create subscription for customer
        let price_id = std::env::var("STRIPE_PRICE_ID").expect("Missing STRIPE_PRICE_ID in env");
        let mut params = CreateSubscription::new(customer.id.clone());
        params.items = Some(vec![CreateSubscriptionItems {
            price: Some(price_id),
            ..Default::default()
        }]);
        params.expand = &["items", "items.data.price.product", "schedule"];

        let subscription = Subscription::create(&self.client, params).await?;
        let subscription_item_id = subscription.items.data[0].id.clone();

        println!("@@@Created subscription id: {:?}", subscription_item_id);

        Ok((customer.id.to_string(), subscription_item_id.to_string()))
    }

    pub async fn report_api_usage(
        &self,
        subscription_item_id: &str,
    ) -> Result<(), StripeClientError> {
        tracing::info!(
            "Reporting usage to Stripe for subscription_item_id {}",
            subscription_item_id
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
                subscription_item_id
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
                subscription_item_id,
                status,
                response_text
            );
            return Ok(());
        }

        tracing::info!(
            "Successfully created usage record for subscription item {}",
            subscription_item_id
        );
        Ok(())
    }
}

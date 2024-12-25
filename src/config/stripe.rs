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
        customer_id: &str,
    ) -> Result<(), StripeClientError> {
        let event_name =
            std::env::var("STRIPE_EVENT_NAME").expect("Missing STRIPE_EVENT_NAME in env");

        println!("@@@Reporting usage to Stripe for stripe_customer_id {}", customer_id);
        let request_body = MeterEventRequest {
            event_name,
            identifier: Uuid::new_v4(),
            timestamp: Utc::now().to_rfc3339(),
            payload: MeterEventPayload {
                stripe_customer_id: customer_id.to_string(),
            },
        };

        let secret_key =
            std::env::var("STRIPE_SECRET_KEY").expect("Missing STRIPE_SECRET_KEY in env");

        let http_client = HttpClient::new();
        let resp = http_client
            .post("https://api.stripe.com/v2/billing/meter_events")
            .header("Stripe-Version", "2024-12-18.acacia")
            .basic_auth(&self.secret_key, Some(secret_key))
            .json(&request_body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_body = resp.text().await?;
            tracing::error!(
                "Failed to create meter event for sub item, body: {}",
                error_body
            );
            return Err(StripeClientError::InvalidRequestError(error_body));
        }

        tracing::info!(
            "Successfully created meter event for count 1",
        );
        Ok(())
    }
}

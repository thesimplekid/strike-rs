//! Create webhook

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::Strike;

/// Webhook state
#[derive(Debug, Clone)]
pub struct WebhookState {
    webhook_secret: String,
    sender: tokio::sync::mpsc::Sender<String>,
}

/// Webhook data
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookRequest {
    #[serde(rename = "webhookUrl")]
    webhook_url: String,
    #[serde(rename = "webhookVersion")]
    webhook_version: String,
    secret: String,
    enabled: bool,
    #[serde(rename = "eventTypes")]
    event_types: Vec<String>,
}

impl Strike {
    /// Create invoice webhook
    pub async fn create_invoice_webhook(
        &self,
        base_url: &str,
        webhook_endpoint: &str,
        sender: tokio::sync::mpsc::Sender<String>,
    ) -> anyhow::Result<Router> {
        let url = self.base_url.join("/v1/subscriptions")?;

        let secret: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        let state = WebhookState {
            sender,
            webhook_secret: secret.clone(),
        };

        let router = Router::new()
            .route(webhook_endpoint, post(handle_invoice))
            .with_state(state);

        let subscription = WebhookRequest {
            webhook_url: format!("{}{}", base_url, webhook_endpoint),
            webhook_version: "v1".to_string(),
            secret,
            enabled: true,
            event_types: vec!["invoice.updated".to_string()],
        };

        self.make_post(url, Some(serde_json::to_value(subscription)?))
            .await?;

        Ok(router)
    }
}

/// Webhook data
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebHookData {
    /// Entity Id
    #[serde(rename = "entityId")]
    entity_id: String,
    /// Changes
    changes: Vec<String>,
}

/// Webhook Response
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
struct WebHookResponse {
    /// Webhook id
    id: String,
    /// Event type
    #[serde(rename = "eventType")]
    event_type: String,
    /// Webhook version
    #[serde(rename = "webhookVersion")]
    webhook_version: String,
    /// Webhook data
    data: WebHookData,
    /// Created
    created: String,
    /// Delivery Success
    #[serde(rename = "deliverySuccess")]
    delivery_success: bool,
}

async fn handle_invoice(
    State(state): State<WebhookState>,
    Json(payload): Json<WebHookResponse>,
) -> StatusCode {
    // TODO: Verify webhook response

    let _secret = state.webhook_secret;

    if let Err(err) = state.sender.send(payload.data.entity_id).await {
        log::warn!("Could not send on channel: {}", err);
    }
    StatusCode::OK
}

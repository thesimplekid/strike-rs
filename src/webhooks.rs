//! Create webhook

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use rand::distributions::Alphanumeric;
use rand::Rng;
use ring::hmac;
use serde::{Deserialize, Serialize};

use crate::{hex, Strike};

/// Webhook state
#[derive(Debug, Clone)]
pub struct WebhookState {
    webhook_secret: String,
    sender: tokio::sync::mpsc::Sender<String>,
}

/// Webhook data
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookRequest {
    webhook_url: String,
    webhook_version: String,
    secret: String,
    enabled: bool,
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
#[serde(rename_all = "camelCase")]
pub struct WebHookData {
    /// Entity Id
    entity_id: String,
    /// Changes
    changes: Vec<String>,
}

/// Webhook Response
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebHookResponse {
    /// Webhook id
    id: String,
    /// Event type
    event_type: String,
    /// Webhook version
    webhook_version: String,
    /// Webhook data
    data: WebHookData,
    /// Created
    created: String,
    /// Delivery Success
    delivery_success: bool,
}

// Function to compute HMAC SHA-256
fn compute_hmac(content: &[u8], secret: &[u8]) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
    let tag = hmac::sign(&key, content);
    hex::encode(tag.as_ref())
}

// Function to get raw body as bytes
fn get_raw_body<T: serde::Serialize>(body: &T) -> Vec<u8> {
    let json_str = serde_json::to_string(body).expect("Failed to serialize body to JSON");
    json_str.into_bytes()
}

// Function to verify request signature
fn verify_request_signature<T: serde::Serialize>(
    request_signature: &str,
    body: &T,
    secret: &[u8],
) -> bool {
    let content_signature = compute_hmac(&get_raw_body(body), secret);
    hmac::verify(
        &hmac::Key::new(hmac::HMAC_SHA256, secret),
        request_signature.as_bytes(),
        content_signature.as_bytes(),
    )
    .is_ok()
}

async fn handle_invoice(
    headers: HeaderMap,
    State(state): State<WebhookState>,
    Json(payload): Json<WebHookResponse>,
) -> Result<StatusCode, StatusCode> {
    let signature = headers
        .get("X-Webhook-Signature")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let secret = state.webhook_secret;
    if !verify_request_signature(signature, &payload, secret.as_bytes()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Err(err) = state.sender.send(payload.data.entity_id).await {
        log::warn!("Could not send on channel: {}", err);
    }
    Ok(StatusCode::OK)
}

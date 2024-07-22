//! Create webhook

use anyhow::anyhow;
use async_trait::async_trait;
use axum::body::Bytes;
use axum::extract::{FromRef, FromRequest, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
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
    pub async fn create_invoice_webhook_router(
        &self,
        webhook_endpoint: &str,
        sender: tokio::sync::mpsc::Sender<String>,
    ) -> anyhow::Result<Router> {
        let state = WebhookState {
            sender,
            webhook_secret: self.webhook_secret.clone(),
        };

        let router = Router::new()
            .route(webhook_endpoint, post(handle_invoice))
            .with_state(state);

        Ok(router)
    }

    /// Subscribe to invoice webhook
    pub async fn subscribe_to_invoice_webhook(&self, webhook_url: String) -> anyhow::Result<()> {
        let url = self.base_url.join("/v1/subscriptions")?;

        let subscription = WebhookRequest {
            webhook_url,
            webhook_version: "v1".to_string(),
            secret: self.webhook_secret.clone(),
            enabled: true,
            event_types: vec!["invoice.updated".to_string()],
        };

        let res = self
            .make_post(url, Some(serde_json::to_value(subscription)?))
            .await?;

        log::debug!("Webhook subscription: {}", res);

        Ok(())
    }
}

fn do_thing_with_request_body(
    bytes: Bytes,
    secret: &str,
    signature: &str,
) -> Result<(), StatusCode> {
    let string = String::from_utf8(bytes.to_vec()).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    verify_request_signature(signature, &string, secret.as_bytes())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(())
}

// extractor that shows how to consume the request body upfront
struct BufferRequestBody(Bytes);

// we must implement `FromRequest` (and not `FromRequestParts`) to consume the
// body
#[async_trait]
impl<S> FromRequest<S> for BufferRequestBody
where
    WebhookState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let headers = req.headers().clone();

        let body = Bytes::from_request(req, state)
            .await
            .map_err(|err| err.into_response())?;

        let signature = headers
            .get("X-Webhook-Signature")
            .ok_or(Self::Rejection::default())?
            .to_str()
            .map_err(|_| Self::Rejection::default())?;

        let state = WebhookState::from_ref(state);

        do_thing_with_request_body(body.clone(), &state.webhook_secret, signature).unwrap();

        Ok(Self(body))
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
    delivery_success: Option<bool>,
}

// Function to compute HMAC SHA-256
fn compute_hmac(content: &[u8], secret: &[u8]) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
    let tag = hmac::sign(&key, content);
    hex::encode(tag.as_ref())
}

// Function to verify request signature
fn verify_request_signature(
    request_signature: &str,
    body: &str,
    secret: &[u8],
) -> anyhow::Result<()> {
    let content_signature = compute_hmac(body.as_bytes(), secret);
    Ok(hmac::verify(
        &hmac::Key::new(hmac::HMAC_SHA256, secret),
        request_signature.as_bytes(),
        content_signature.as_bytes(),
    )
    .map_err(|_| anyhow!("Invalid signature"))?)
}

async fn handle_invoice(
    headers: HeaderMap,
    State(state): State<WebhookState>,
    Json(payload): Json<String>,
) -> Result<StatusCode, StatusCode> {
    let signature = headers
        .get("X-Webhook-Signature")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let webhook_response: WebHookResponse =
        serde_json::from_str(&payload).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    log::debug!(
        "Received webhook update for: {}",
        webhook_response.data.entity_id
    );

    let secret = state.webhook_secret;
    if let Err(_err) = verify_request_signature(signature, &payload, secret.as_bytes()) {
        log::warn!("Signature verification failed");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Err(err) = state.sender.send(webhook_response.data.entity_id).await {
        log::warn!("Could not send on channel: {}", err);
    }
    Ok(StatusCode::OK)
}

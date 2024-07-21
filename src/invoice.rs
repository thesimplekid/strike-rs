//! Handle invoice creation

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Amount, ConversionRate, InvoiceState, Strike};

/// Invoice Request
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceRequest {
    /// Correlation ID
    pub correlation_id: Option<String>,
    /// Invoice description
    pub description: Option<String>,
    /// Invoice [`Amount`]
    pub amount: Amount,
}

/// Invoice Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceResponse {
    /// Invoice ID
    pub invoice_id: String,
    /// Invoice [`Amount`]
    pub amount: Amount,
    /// Invoice State
    pub state: InvoiceState,
    /// Created timestamp
    pub created: String,
    /// Invoice Description
    pub description: Option<String>,
    /// Isser ID
    pub issuer_id: String,
    /// Receiver ID
    pub receiver_id: String,
}

/// Invoice Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceQuoteResponse {
    /// Invoice Quote ID
    pub quote_id: String,
    /// Invoice description
    pub description: Option<String>,
    /// Bolt11 invoice
    pub ln_invoice: String,
    /// Onchain Address
    pub onchain_address: Option<String>,
    /// Expiration of quote
    pub expiration: String,
    /// Experition in secs
    pub expiration_in_sec: u64,
    /// Source Amount
    pub source_amount: Amount,
    /// Target Amount
    pub target_amount: Amount,
    /// Conversion Rate
    pub conversion_rate: ConversionRate,
}

impl Strike {
    /// Create Invoice
    pub async fn create_invoice(&self, invoice_request: InvoiceRequest) -> Result<InvoiceResponse> {
        let url = self.base_url.join("/v1/invoices")?;

        let res = self
            .make_post(url, Some(serde_json::to_value(invoice_request)?))
            .await?;

        Ok(serde_json::from_value(res)?)
    }

    /// Find Invoice
    pub async fn find_invoice(&self, invoice_id: &str) -> Result<InvoiceResponse> {
        let url = self.base_url.join("/v1/invoices/")?.join(&invoice_id)?;

        let res = self.make_get(url).await?;

        Ok(serde_json::from_value(res)?)
    }

    /// Invoice quote
    pub async fn invoice_quote(&self, invoice_id: String) -> Result<InvoiceQuoteResponse> {
        let url = self
            .base_url
            .join(&format!("/v1/invoices/{invoice_id}/quote"))?;

        let res = self.make_post(url, None).await?;
        Ok(serde_json::from_value(res)?)
    }
}

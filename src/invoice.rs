//! Handle invoice creation

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Amount, ConversionRate, InvoiceState, Strike};

/// Invoice Request
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InvoiceRequest {
    /// Correlation ID
    #[serde(rename = "correlationId")]
    pub correlation_id: Option<String>,
    /// Invoice description
    pub description: Option<String>,
    /// Invoice [`Amount`]
    pub amount: Amount,
}

/// Invoice Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InvoiceResponse {
    /// Invoice ID
    #[serde(rename = "invoiceId")]
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
    #[serde(rename = "issuerId")]
    pub issuer_id: String,
    /// Receiver ID
    #[serde(rename = "receiverId")]
    pub receiver_id: String,
}

/// Invoice Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InvoiceQuoteResponse {
    /// Invoice Quote ID
    #[serde(rename = "quoteId")]
    pub quote_id: String,
    /// Invoice description
    pub description: Option<String>,
    /// Bolt11 invoice
    #[serde(rename = "lnInvoice")]
    pub ln_invoice: String,
    /// Onchain Address
    #[serde(rename = "onchainAddress")]
    pub onchain_address: Option<String>,
    /// Expiration of quote
    pub expiration: String,
    /// Experition in secs
    #[serde(rename = "expirationInSec")]
    pub expiration_in_sec: u64,
    /// Source Amount
    #[serde(rename = "sourceAmount")]
    pub source_amount: Amount,
    #[serde(rename = "targetAmount")]
    /// Target Amount
    pub target_amount: Amount,
    /// Conversion Rate
    #[serde(rename = "conversionRate")]
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

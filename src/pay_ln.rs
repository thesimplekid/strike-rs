//! Pay Ln

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Amount, ConversionRate, Currency, InvoiceState, Strike};

/// Pay Invoice Request
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PayInvoiceQuoteRequest {
    /// Bolt11 Invoice
    #[serde(rename = "lnInvoice")]
    pub ln_invoice: String,
    /// Source Currency
    #[serde(rename = "sourceCurrency")]
    pub source_currency: Currency,
}

/// Pay Invoice Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PayInvoiceQuoteResponse {
    /// Payment quote Id
    #[serde(rename = "paymentQuoteId")]
    pub payment_quote_id: String,
    /// Description
    pub description: String,
    /// Quote valid till
    #[serde(rename = "validUntil")]
    pub valid_until: String,
    /// Conversion quote
    #[serde(rename = "conversionRate")]
    pub conversion_rate: Option<ConversionRate>,
    /// Amount
    pub amount: Amount,
    /// Network fee
    #[serde(rename = "lightningNetworkFee")]
    pub lightning_network_fee: Amount,
    /// Total amount including fee
    #[serde(rename = "totalAmount")]
    pub total_amount: Amount,
}

/// Pay Quote Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InvoicePaymentResponse {
    /// Payment id
    #[serde(rename = "paymentId")]
    pub payment_id: String,
    /// Invoice state
    pub state: InvoiceState,
    /// Completed time stamp
    pub completed: Option<String>,
    /// Conversion quote
    #[serde(rename = "conversionRate")]
    pub conversion_rate: Option<ConversionRate>,
    /// Amount
    pub amount: Amount,
    /// Network fee
    #[serde(rename = "lightningNetworkFee")]
    pub lightning_network_fee: Amount,
    /// Total amount including fee
    #[serde(rename = "totalAmount")]
    pub total_amount: Amount,
}

impl Strike {
    /// Create Payment Quote
    pub async fn payment_quote(
        &self,
        quote_request: PayInvoiceQuoteRequest,
    ) -> Result<PayInvoiceQuoteResponse> {
        let url = self.base_url.join("/v1/payment-quotes/lightning")?;

        let res = self
            .make_post(url, Some(serde_json::to_value(quote_request)?))
            .await?;

        Ok(serde_json::from_value(res)?)
    }

    /// Execute quote to pay invoice
    pub async fn pay_quote(&self, payment_quote_id: &str) -> Result<InvoicePaymentResponse> {
        let url = self
            .base_url
            .join(&format!("/v1/payment-quotes/{payment_quote_id}/execute"))?;

        let res = self.make_patch(url).await?;

        Ok(serde_json::from_value(res)?)
    }
}

//! Pay Ln

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Amount, ConversionRate, Currency, InvoiceState, Strike};

/// Pay Invoice Request
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayInvoiceQuoteRequest {
    /// Bolt11 Invoice
    pub ln_invoice: String,
    /// Source Currency
    pub source_currency: Currency,
}

/// Pay Invoice Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayInvoiceQuoteResponse {
    /// Payment quote Id
    pub payment_quote_id: String,
    /// Description
    pub description: Option<String>,
    /// Quote valid till
    pub valid_until: String,
    /// Conversion quote
    pub conversion_rate: Option<ConversionRate>,
    /// Amount
    pub amount: Amount,
    /// Network fee
    pub lightning_network_fee: Amount,
    /// Total amount including fee
    pub total_amount: Amount,
}

/// Pay Quote Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoicePaymentResponse {
    /// Payment id
    pub payment_id: String,
    /// Invoice state
    pub state: InvoiceState,
    /// Completed time stamp
    pub completed: Option<String>,
    /// Conversion quote
    pub conversion_rate: Option<ConversionRate>,
    /// Amount
    pub amount: Amount,
    /// Network fee
    pub lightning_network_fee: Amount,
    /// Total amount including fee
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

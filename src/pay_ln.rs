//! Pay Ln

use anyhow::{bail, Result};
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

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on payment quote");
                log::error!("{}", res);
                bail!("Could not get payment quote")
            }
        }
    }

    /// Execute quote to pay invoice
    pub async fn pay_quote(&self, payment_quote_id: &str) -> Result<InvoicePaymentResponse> {
        let url = self
            .base_url
            .join(&format!("/v1/payment-quotes/{payment_quote_id}/execute"))?;

        let res = self.make_patch(url).await?;

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on payment quote execution");
                log::error!("{}", res);
                bail!("Could not execute payment quote")
            }
        }
    }

    /// Get outgoing payment by payment id
    pub async fn get_outgoing_payment(&self, payment_id: &str) -> Result<InvoicePaymentResponse> {
        let url = self.base_url.join(&format!("/v1/payments/{payment_id}"))?;

        let res = self.make_get(url).await?;

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response getting payment quote");
                log::error!("{}", res);
                bail!("Could not get payment by id")
            }
        }
    }
}

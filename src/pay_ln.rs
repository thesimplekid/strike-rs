//! Pay Ln

use anyhow::{bail, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{Amount, ConversionRate, Currency, Error, InvoiceState, Strike};

/// Pay Invoice Request
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayInvoiceQuoteRequest {
    /// Bolt11 Invoice
    pub ln_invoice: String,
    /// Source Currency
    pub source_currency: Currency,
    /// Amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<RequestAmount>,
}

/// Request amount
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestAmount {
    /// Amount
    pub amount: f32,
    /// Currency
    pub currency: Currency,
    /// Fee Policy
    pub fee_policy: FeePolicy,
}

/// Fee Policy
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FeePolicy {
    /// Fee should be included in the amount
    Inclusive,
    /// Fee should be added on top
    Exclusive,
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
    pub async fn get_outgoing_payment(
        &self,
        payment_id: &str,
    ) -> Result<InvoicePaymentResponse, Error> {
        let url = self
            .base_url
            .join(&format!("/v1/payments/{payment_id}"))
            .map_err(|_| Error::InvalidUrl)?;

        let res = match self.make_get(url).await {
            Ok(res) => res,
            Err(err) => {
                if let Error::ReqwestError(err) = &err {
                    if err.status().unwrap_or_default() == StatusCode::NOT_FOUND {
                        return Err(Error::NotFound);
                    }
                }
                return Err(err);
            }
        };

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(err) => {
                log::error!("Api error response getting payment quote");
                log::error!("{}", res);

                Err(err.into())
            }
        }
    }
}

//! Strike API SDK
//! Rust SDK for <https://strike.me/>
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use std::fmt;
use std::str::FromStr;

use anyhow::{anyhow, bail, Result};
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::{Client, IntoUrl, Url};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub(crate) mod hex;
pub mod invoice;
pub mod pay_ln;
pub mod webhooks;

pub use invoice::*;
pub use pay_ln::*;

/// Strike
#[derive(Debug, Clone)]
pub struct Strike {
    api_key: String,
    base_url: Url,
    client: Client,
    webhook_secret: String,
}

/// Currency unit
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    /// USD
    USD,
    /// EURO
    EUR,
    /// Bitcoin
    BTC,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::USD => write!(f, "USD"),
            Self::EUR => write!(f, "EUR"),
            Self::BTC => write!(f, "BTC"),
        }
    }
}

/// Amount with unit
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Amount {
    /// Currency of amount
    pub currency: Currency,
    /// Value of amount
    #[serde(deserialize_with = "parse_f64_from_string")]
    pub amount: f64,
}

fn parse_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

impl Amount {
    /// Amount from sats
    pub fn from_sats(amount: u64) -> Self {
        Self {
            currency: Currency::BTC,
            amount: amount as f64 / 100_000_000.0,
        }
    }

    /// Unit as sats
    pub fn to_sats(&self) -> Result<u64> {
        match self.currency {
            Currency::BTC => Ok((self.amount * 100_000_000.0) as u64),
            _ => bail!("Unit cannot be converted to sats"),
        }
    }
}

/// Invoice state
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum InvoiceState {
    /// Payment Completed
    Completed,
    /// Invoice paid
    Paid,
    /// Invoice unpaid
    Unpaid,
    /// Invoice pending
    Pending,
}

/// Conversion rate for quote
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ConversionRate {
    /// Amount
    #[serde(deserialize_with = "parse_f64_from_string")]
    pub amount: f64,
    /// Source Unit
    #[serde(rename = "sourceCurrency")]
    pub source_currency: Currency,
    /// Target Unit
    #[serde(rename = "targetCurrency")]
    pub target_currency: Currency,
}

impl Strike {
    /// Create Strike client
    /// # Arguments
    /// * `api_key` - Strike api token
    /// * `url` - Optional Url of nodeless api
    ///
    /// # Example
    /// ```
    /// use strike_rs::Strike;
    /// let client = Strike::new("xxxxxxxxxxx", None).unwrap();
    /// ```
    pub fn new(api_key: &str, api_url: Option<String>) -> Result<Self> {
        let base_url = match api_url {
            Some(url) => Url::from_str(&url)?,
            None => Url::from_str("https://api.strike.me")?,
        };

        let client = reqwest::Client::builder().build()?;
        let secret: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        Ok(Self {
            api_key: api_key.to_string(),
            base_url,
            client,
            webhook_secret: secret,
        })
    }

    async fn make_get<U>(&self, url: U) -> Result<Value>
    where
        U: IntoUrl,
    {
        Ok(self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("accept", "application/json")
            .send()
            .await?
            .json::<Value>()
            .await?)
    }

    async fn make_post<U, T>(&self, url: U, data: Option<T>) -> Result<Value>
    where
        U: IntoUrl,
        T: Serialize,
    {
        let value = match data {
            Some(data) => {
                self.client
                    .post(url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Type", "application/json")
                    .header("accept", "application/json")
                    .json(&data)
                    .send()
                    .await?
                    .json::<Value>()
                    .await?
            }
            None => {
                self.client
                    .post(url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Length", "0")
                    .header("accept", "application/json")
                    .send()
                    .await?
                    .json::<Value>()
                    .await?
            }
        };
        Ok(value)
    }

    async fn make_patch<U>(&self, url: U) -> Result<Value>
    where
        U: IntoUrl,
    {
        Ok(self
            .client
            .patch(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Length", "0")
            .header("accept", "application/json")
            .send()
            .await?
            .json::<Value>()
            .await?)
    }

    async fn make_delete<U>(&self, url: U) -> Result<()>
    where
        U: IntoUrl,
    {
        self.client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|err| anyhow!("Error making delete: {}", err.to_string()))?;

        Ok(())
    }

    /*
    async fn make_put(&self, url: Url, data: Option<Value>) -> Result<Value> {
        let res = self
            .client
            .put(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("accept", "application/json")
            .json(&data)
            .send()
            .await?;
        let res = res.json::<Value>().await?;
        Ok(res)
    }

    */
}

use async_trait::async_trait;
use gravitas_core::types::{ExpiryFilter, OptionContract, OptionType};
use serde::Deserialize;
use std::time::Duration;

use crate::source::{DataSource, FetchError};

pub struct TradierSource {
    client: reqwest::Client,
    token: String,
    base_url: String,
}

impl TradierSource {
    pub fn new(token: String, sandbox: bool) -> Self {
        let base_url = if sandbox {
            "https://sandbox.tradier.com/v1".to_string()
        } else {
            "https://api.tradier.com/v1".to_string()
        };
        Self {
            client: reqwest::Client::new(),
            token,
            base_url,
        }
    }
}

// --- Tradier API response types ---

#[derive(Deserialize)]
struct ExpirationsResponse {
    expirations: Option<Expirations>,
}

#[derive(Deserialize)]
struct Expirations {
    date: Option<DateOrDates>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DateOrDates {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize)]
struct ChainsResponse {
    options: Option<Options>,
}

#[derive(Deserialize)]
struct Options {
    option: Option<OptionOrOptions>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OptionOrOptions {
    Single(Box<TradierOption>),
    Multiple(Vec<TradierOption>),
}

#[derive(Deserialize)]
struct TradierOption {
    strike: f64,
    option_type: String, // "call" or "put"
    open_interest: Option<u64>,
    bid: Option<f64>,
    ask: Option<f64>,
    expiration_date: String,
    greeks: Option<TradierGreeks>,
}

#[derive(Deserialize)]
struct TradierGreeks {
    mid_iv: Option<f64>,
    smv_vol: Option<f64>,
}

#[derive(Deserialize)]
struct QuotesResponse {
    quotes: Option<Quotes>,
}

#[derive(Deserialize)]
struct Quotes {
    quote: Option<QuoteOrQuotes>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum QuoteOrQuotes {
    Single(Box<TradierQuote>),
    Multiple(Vec<TradierQuote>),
}

#[derive(Deserialize)]
struct TradierQuote {
    last: Option<f64>,
}

// --- Implementation ---

#[async_trait]
impl DataSource for TradierSource {
    async fn fetch_options_chain(
        &self,
        symbol: &str,
        expiry_filter: Option<ExpiryFilter>,
    ) -> Result<Vec<OptionContract>, FetchError> {
        let expirations = self.fetch_expirations(symbol).await?;
        let today = chrono::Utc::now().date_naive();

        let filtered: Vec<&str> = match &expiry_filter {
            None | Some(ExpiryFilter::NextN(4)) => expirations.iter().take(4).map(|s| s.as_str()).collect(),
            Some(ExpiryFilter::NextN(n)) => expirations.iter().take(*n as usize).map(|s| s.as_str()).collect(),
            Some(ExpiryFilter::ZeroDte) => {
                let today_str = today.format("%Y-%m-%d").to_string();
                expirations.iter().filter(|d| **d == today_str).map(|s| s.as_str()).collect()
            }
            Some(ExpiryFilter::DateRange(from, to)) => {
                let from_str = from.format("%Y-%m-%d").to_string();
                let to_str = to.format("%Y-%m-%d").to_string();
                expirations
                    .iter()
                    .filter(|d| d.as_str() >= from_str.as_str() && d.as_str() <= to_str.as_str())
                    .map(|s| s.as_str())
                    .collect()
            }
        };

        let mut contracts = Vec::new();
        for expiry in filtered {
            let mut chain = self.fetch_chain_for_expiry(symbol, expiry).await?;
            contracts.append(&mut chain);
        }

        Ok(contracts)
    }

    async fn fetch_spot_price(&self, symbol: &str) -> Result<f64, FetchError> {
        let url = format!("{}/markets/quotes", self.base_url);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .query(&[("symbols", symbol)])
            .send()
            .await
            .map_err(FetchError::Http)?;

        if resp.status() == 429 {
            return Err(FetchError::RateLimited {
                retry_after: Duration::from_secs(60),
            });
        }
        if resp.status() == 401 {
            return Err(FetchError::AuthError);
        }
        if !resp.status().is_success() {
            return Err(FetchError::ApiDown(format!("status {}", resp.status())));
        }

        let body: QuotesResponse = resp.json().await.map_err(FetchError::Http)?;
        let price = body
            .quotes
            .and_then(|q| q.quote)
            .and_then(|q| match q {
                QuoteOrQuotes::Single(q) => q.last,
                QuoteOrQuotes::Multiple(qs) => qs.first().and_then(|q| q.last),
            })
            .ok_or_else(|| FetchError::ParseError("no quote data".to_string()))?;

        Ok(price)
    }

    fn name(&self) -> &'static str {
        "tradier"
    }

    fn has_realtime(&self) -> bool {
        false
    }

    fn refresh_interval(&self) -> Duration {
        Duration::from_secs(60)
    }
}

impl TradierSource {
    async fn fetch_expirations(&self, symbol: &str) -> Result<Vec<String>, FetchError> {
        let url = format!("{}/markets/options/expirations", self.base_url);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .query(&[("symbol", symbol)])
            .send()
            .await
            .map_err(FetchError::Http)?;

        if resp.status() == 429 {
            return Err(FetchError::RateLimited {
                retry_after: Duration::from_secs(60),
            });
        }
        if !resp.status().is_success() {
            return Err(FetchError::ApiDown(format!("status {}", resp.status())));
        }

        let body: ExpirationsResponse = resp.json().await.map_err(FetchError::Http)?;
        let dates = body
            .expirations
            .and_then(|e| e.date)
            .map(|d| match d {
                DateOrDates::Single(s) => vec![s],
                DateOrDates::Multiple(v) => v,
            })
            .unwrap_or_default();

        Ok(dates)
    }

    async fn fetch_chain_for_expiry(
        &self,
        symbol: &str,
        expiration: &str,
    ) -> Result<Vec<OptionContract>, FetchError> {
        let url = format!("{}/markets/options/chains", self.base_url);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .query(&[
                ("symbol", symbol),
                ("expiration", expiration),
                ("greeks", "true"),
            ])
            .send()
            .await
            .map_err(FetchError::Http)?;

        if resp.status() == 429 {
            return Err(FetchError::RateLimited {
                retry_after: Duration::from_secs(60),
            });
        }
        if !resp.status().is_success() {
            return Err(FetchError::ApiDown(format!("status {}", resp.status())));
        }

        let body: ChainsResponse = resp.json().await.map_err(FetchError::Http)?;
        let tradier_options = body
            .options
            .and_then(|o| o.option)
            .map(|o| match o {
                OptionOrOptions::Single(opt) => vec![*opt],
                OptionOrOptions::Multiple(opts) => opts,
            })
            .unwrap_or_default();

        let contracts: Vec<OptionContract> = tradier_options
            .into_iter()
            .filter_map(|opt| {
                let option_type = match opt.option_type.as_str() {
                    "call" => OptionType::Call,
                    "put" => OptionType::Put,
                    _ => return None,
                };

                let iv = opt
                    .greeks
                    .as_ref()
                    .and_then(|g| g.mid_iv.filter(|v| *v > 0.0))
                    .or_else(|| opt.greeks.as_ref().and_then(|g| g.smv_vol))
                    .unwrap_or(0.0);

                if iv <= 0.0 {
                    return None; // skip contracts with no IV data
                }

                let expiry = chrono::NaiveDate::parse_from_str(&opt.expiration_date, "%Y-%m-%d")
                    .ok()?;

                Some(OptionContract {
                    strike: opt.strike,
                    expiry,
                    option_type,
                    open_interest: opt.open_interest.unwrap_or(0),
                    implied_volatility: iv,
                    bid: opt.bid.unwrap_or(0.0),
                    ask: opt.ask.unwrap_or(0.0),
                })
            })
            .collect();

        Ok(contracts)
    }
}

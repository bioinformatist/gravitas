use async_trait::async_trait;
use gravitas_core::types::{ExpiryFilter, OptionContract};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("rate limited, retry after {retry_after:?}")]
    RateLimited { retry_after: Duration },
    #[error("API down: {0}")]
    ApiDown(String),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("authentication error")]
    AuthError,
    #[error("request timeout")]
    Timeout,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[async_trait]
pub trait DataSource: Send + Sync {
    async fn fetch_options_chain(
        &self,
        symbol: &str,
        expiry_filter: Option<ExpiryFilter>,
    ) -> Result<Vec<OptionContract>, FetchError>;

    async fn fetch_spot_price(&self, symbol: &str) -> Result<f64, FetchError>;

    fn name(&self) -> &'static str;
    fn has_realtime(&self) -> bool;
    fn refresh_interval(&self) -> Duration;
}

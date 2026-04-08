use async_trait::async_trait;
use gravitas_core::types::{ExpiryFilter, OptionContract, OptionType};
use std::time::Duration;

use crate::source::{DataSource, FetchError};

/// Deterministic mock data source for testing.
pub struct MockSource;

impl MockSource {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataSource for MockSource {
    async fn fetch_options_chain(
        &self,
        _symbol: &str,
        _expiry_filter: Option<ExpiryFilter>,
    ) -> Result<Vec<OptionContract>, FetchError> {
        let expiry = chrono::Utc::now().date_naive() + chrono::Duration::days(30);
        let strikes = [510.0, 515.0, 520.0, 525.0, 530.0, 535.0, 540.0];

        let mut contracts = Vec::new();
        for &strike in &strikes {
            contracts.push(OptionContract {
                strike,
                expiry,
                option_type: OptionType::Call,
                open_interest: (10000.0 + (530.0 - strike).abs() * 500.0) as u64,
                implied_volatility: 0.20 + (strike - 525.0).abs() * 0.001,
                bid: 1.0,
                ask: 1.5,
            });
            contracts.push(OptionContract {
                strike,
                expiry,
                option_type: OptionType::Put,
                open_interest: (10000.0 + (520.0 - strike).abs() * 500.0) as u64,
                implied_volatility: 0.22 + (strike - 525.0).abs() * 0.001,
                bid: 1.0,
                ask: 1.5,
            });
        }

        Ok(contracts)
    }

    async fn fetch_spot_price(&self, _symbol: &str) -> Result<f64, FetchError> {
        Ok(525.0)
    }

    fn name(&self) -> &'static str {
        "mock"
    }

    fn has_realtime(&self) -> bool {
        false
    }

    fn refresh_interval(&self) -> Duration {
        Duration::from_secs(60)
    }
}

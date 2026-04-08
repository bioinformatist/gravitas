use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpiryFilter {
    NextN(u32),
    DateRange(NaiveDate, NaiveDate),
    ZeroDte,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContract {
    pub strike: f64,
    pub expiry: NaiveDate,
    pub option_type: OptionType,
    pub open_interest: u64,
    pub implied_volatility: f64,
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikeGex {
    pub strike: f64,
    pub call_gex: f64,
    pub put_gex: f64,
    pub net_gex: f64,
    pub vanna: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GexResult {
    pub symbol: String,
    pub spot_price: f64,
    pub computed_at: chrono::DateTime<chrono::Utc>,
    pub strikes: Vec<StrikeGex>,
    pub zero_gamma_levels: Vec<f64>,
    pub nearest_zgl: f64,
    pub total_net_gex: f64,
    pub is_negative_gex_regime: bool,
}

use crate::types::{GexResult, OptionContract};

/// Recompute GEX at a hypothetical spot price.
/// This is used by the WASM UI for interactive scenario slider.
pub fn compute_scenario(
    symbol: &str,
    contracts: &[OptionContract],
    hypothetical_spot: f64,
    risk_free_rate: f64,
) -> GexResult {
    crate::gex::compute_gex(symbol, contracts, hypothetical_spot, risk_free_rate)
}

/// Compute GEX across a range of hypothetical prices.
/// Returns a vec of (price, total_net_gex) for charting the GEX curve.
pub fn compute_scenario_range(
    symbol: &str,
    contracts: &[OptionContract],
    spot_price: f64,
    risk_free_rate: f64,
    range_pct: f64,
    steps: usize,
) -> Vec<(f64, f64)> {
    let low = spot_price * (1.0 - range_pct / 100.0);
    let high = spot_price * (1.0 + range_pct / 100.0);
    let step_size = (high - low) / steps as f64;

    (0..=steps)
        .map(|i| {
            let price = low + step_size * i as f64;
            let result = crate::gex::compute_gex(symbol, contracts, price, risk_free_rate);
            (price, result.total_net_gex)
        })
        .collect()
}

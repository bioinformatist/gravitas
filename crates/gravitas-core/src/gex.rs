use crate::types::{GexResult, OptionContract, StrikeGex};
use ordered_float::OrderedFloat;
use regit_blackscholes::greeks::compute_greeks;
use regit_blackscholes::types::{OptionParams, OptionType as BsOptionType};

/// Compute GEX for a set of option contracts at a given spot price.
pub fn compute_gex(
    symbol: &str,
    contracts: &[OptionContract],
    spot_price: f64,
    risk_free_rate: f64,
) -> GexResult {
    let now = chrono::Utc::now();
    let today = now.date_naive();

    // Aggregate GEX per strike: (call_gex, put_gex, vanna)
    let mut strike_map: std::collections::BTreeMap<OrderedFloat<f64>, (f64, f64, f64)> =
        std::collections::BTreeMap::new();

    for contract in contracts {
        let days_to_expiry = (contract.expiry - today).num_days();
        if days_to_expiry <= 0 {
            continue;
        }
        let t = days_to_expiry as f64 / 365.0;

        let option_type = match contract.option_type {
            crate::types::OptionType::Call => BsOptionType::Call,
            crate::types::OptionType::Put => BsOptionType::Put,
        };

        let params = OptionParams {
            option_type,
            spot: spot_price,
            strike: contract.strike,
            rate: risk_free_rate,
            div_yield: 0.0,
            vol: contract.implied_volatility,
            time: t,
        };

        let greeks = match compute_greeks(&params) {
            Ok(g) => g,
            Err(_) => continue,
        };

        let oi = contract.open_interest as f64;
        let gex = greeks.gamma * oi * 100.0 * spot_price * spot_price;
        let vanna_exp = greeks.vanna * oi * 100.0;

        let key = OrderedFloat(contract.strike);
        let entry = strike_map.entry(key).or_insert((0.0, 0.0, 0.0));

        match contract.option_type {
            crate::types::OptionType::Call => {
                entry.0 += gex;
                entry.2 += vanna_exp;
            }
            crate::types::OptionType::Put => {
                entry.1 += gex;
                entry.2 -= vanna_exp;
            }
        }
    }

    let mut strikes: Vec<StrikeGex> = strike_map
        .into_iter()
        .map(|(k, (call_gex, put_gex, vanna))| StrikeGex {
            strike: k.into_inner(),
            call_gex,
            put_gex,
            net_gex: call_gex - put_gex,
            vanna,
        })
        .collect();

    strikes.sort_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap());

    let total_net_gex: f64 = strikes.iter().map(|s| s.net_gex).sum();

    let zero_gamma_levels = find_zero_gamma_levels(&strikes);

    let nearest_zgl = zero_gamma_levels
        .iter()
        .copied()
        .min_by(|a, b| {
            (a - spot_price)
                .abs()
                .partial_cmp(&(b - spot_price).abs())
                .unwrap()
        })
        .unwrap_or(spot_price);

    GexResult {
        symbol: symbol.to_string(),
        spot_price,
        computed_at: now,
        strikes,
        zero_gamma_levels,
        nearest_zgl,
        total_net_gex,
        is_negative_gex_regime: total_net_gex < 0.0,
    }
}

/// Find prices where net_gex crosses zero by linear interpolation
/// between adjacent strikes.
fn find_zero_gamma_levels(strikes: &[StrikeGex]) -> Vec<f64> {
    let mut zgls = Vec::new();

    for window in strikes.windows(2) {
        let a = &window[0];
        let b = &window[1];

        if a.net_gex.signum() != b.net_gex.signum() && a.net_gex != 0.0 && b.net_gex != 0.0 {
            let ratio = a.net_gex.abs() / (a.net_gex.abs() + b.net_gex.abs());
            let zgl = a.strike + ratio * (b.strike - a.strike);
            zgls.push(zgl);
        }
    }

    zgls
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OptionContract, OptionType};

    fn mock_contracts() -> Vec<OptionContract> {
        let expiry = chrono::Utc::now().date_naive() + chrono::Duration::days(30);
        vec![
            OptionContract {
                strike: 520.0,
                expiry,
                option_type: OptionType::Call,
                open_interest: 10000,
                implied_volatility: 0.20,
                bid: 5.0,
                ask: 5.5,
            },
            OptionContract {
                strike: 520.0,
                expiry,
                option_type: OptionType::Put,
                open_interest: 8000,
                implied_volatility: 0.22,
                bid: 4.0,
                ask: 4.5,
            },
            OptionContract {
                strike: 530.0,
                expiry,
                option_type: OptionType::Call,
                open_interest: 15000,
                implied_volatility: 0.18,
                bid: 2.0,
                ask: 2.5,
            },
            OptionContract {
                strike: 530.0,
                expiry,
                option_type: OptionType::Put,
                open_interest: 20000,
                implied_volatility: 0.25,
                bid: 8.0,
                ask: 8.5,
            },
        ]
    }

    #[test]
    fn test_compute_gex_basic() {
        let contracts = mock_contracts();
        let result = compute_gex("SPY", &contracts, 525.0, 0.05);

        assert_eq!(result.symbol, "SPY");
        assert_eq!(result.spot_price, 525.0);
        assert!(!result.strikes.is_empty());

        for strike in &result.strikes {
            assert!(strike.call_gex > 0.0, "call_gex should be positive");
            assert!(strike.put_gex > 0.0, "put_gex should be positive");
        }
    }

    #[test]
    fn test_zero_gamma_levels() {
        let strikes = vec![
            StrikeGex {
                strike: 500.0,
                call_gex: 100.0,
                put_gex: 200.0,
                net_gex: -100.0,
                vanna: 0.0,
            },
            StrikeGex {
                strike: 510.0,
                call_gex: 200.0,
                put_gex: 100.0,
                net_gex: 100.0,
                vanna: 0.0,
            },
        ];

        let zgls = find_zero_gamma_levels(&strikes);
        assert_eq!(zgls.len(), 1);
        assert!((zgls[0] - 505.0).abs() < 0.01);
    }
}

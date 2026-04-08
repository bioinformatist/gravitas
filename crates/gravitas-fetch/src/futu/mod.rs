//! Futu OpenD data source adapter.
//!
//! Two-step flow:
//! 1. GetOptionChain (3209) → list of option contract securities
//! 2. GetBasicQot (3004) on each contract → OI, IV, price

pub mod client;
pub mod codec;
pub mod proto;

use async_trait::async_trait;
use gravitas_core::types::{ExpiryFilter, OptionContract, OptionType};
use prost::Message;
use std::time::Duration;
use tokio::sync::OnceCell;

use crate::source::{DataSource, FetchError};
use client::FutuClient;
use proto::{get_basic_qot, get_option_chain, Security, MARKET_US};

const PROTO_GET_BASIC_QOT: u32 = 3004;
const PROTO_GET_OPTION_CHAIN: u32 = 3209;

pub struct FutuSource {
    client: OnceCell<FutuClient>,
    host: String,
    port: u16,
}

impl FutuSource {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            client: OnceCell::new(),
            host,
            port,
        }
    }

    async fn ensure_connected(&self) -> Result<&FutuClient, FetchError> {
        self.client
            .get_or_try_init(|| async {
                FutuClient::connect(&self.host, self.port).await
            })
            .await
    }

    /// Convert gravitas symbol "SPY" → Futu Security { market: 11, code: "SPY" }
    fn to_security(symbol: &str) -> Security {
        Security {
            market: MARKET_US,
            code: symbol.to_string(),
        }
    }
}

#[async_trait]
impl DataSource for FutuSource {
    async fn fetch_spot_price(&self, symbol: &str) -> Result<f64, FetchError> {
        let client = self.ensure_connected().await?;

        let req = get_basic_qot::Request {
            c2s: get_basic_qot::C2s {
                security_list: vec![Self::to_security(symbol)],
            },
        };

        let resp: get_basic_qot::Response = client
            .request(PROTO_GET_BASIC_QOT, &req.encode_to_vec())
            .await?;

        if resp.ret_type != 0 {
            return Err(FetchError::ApiDown(format!(
                "GetBasicQot failed: {:?}",
                resp.ret_msg
            )));
        }

        let s2c = resp.s2c.ok_or_else(|| FetchError::ParseError("no s2c".into()))?;
        let qot = s2c
            .basic_qot_list
            .first()
            .ok_or_else(|| FetchError::ParseError("empty basic_qot_list".into()))?;

        Ok(qot.cur_price)
    }

    async fn fetch_options_chain(
        &self,
        symbol: &str,
        expiry_filter: Option<ExpiryFilter>,
    ) -> Result<Vec<OptionContract>, FetchError> {
        let client = self.ensure_connected().await?;

        // Step 1: Get option chain (static info)
        let today = chrono::Utc::now().date_naive();
        let (begin, end) = match &expiry_filter {
            Some(ExpiryFilter::ZeroDte) => (today, today),
            Some(ExpiryFilter::DateRange(from, to)) => (*from, *to),
            _ => (today, today + chrono::Duration::days(90)),
        };

        let chain_req = get_option_chain::Request {
            c2s: get_option_chain::C2s {
                owner: Self::to_security(symbol),
                begin_time: begin.format("%Y-%m-%d").to_string(),
                end_time: end.format("%Y-%m-%d").to_string(),
            },
        };

        let chain_resp: get_option_chain::Response = client
            .request(PROTO_GET_OPTION_CHAIN, &chain_req.encode_to_vec())
            .await?;

        if chain_resp.ret_type != 0 {
            return Err(FetchError::ApiDown(format!(
                "GetOptionChain failed: {:?}",
                chain_resp.ret_msg
            )));
        }

        let s2c = chain_resp
            .s2c
            .ok_or_else(|| FetchError::ParseError("no s2c".into()))?;

        // Collect all option securities
        let mut option_securities: Vec<(Security, f64, String, OptionType)> = Vec::new();

        for chain in &s2c.option_chain {
            for item in &chain.option {
                for (info, opt_type) in [
                    (&item.call, OptionType::Call),
                    (&item.put, OptionType::Put),
                ] {
                    if let Some(info) = info {
                        if let Some(opt_data) = &info.option_ex_data {
                            if !opt_data.suspend {
                                option_securities.push((
                                    info.basic.security.clone(),
                                    opt_data.strike_price,
                                    opt_data.strike_time.clone(),
                                    opt_type,
                                ));
                            }
                        }
                    }
                }
            }
        }

        if option_securities.is_empty() {
            return Ok(vec![]);
        }

        // Step 2: Get real-time data for all option contracts (batch)
        // Futu limits ~200 securities per request, batch if needed
        let mut contracts = Vec::new();
        for chunk in option_securities.chunks(200) {
            let sec_list: Vec<Security> = chunk.iter().map(|(s, _, _, _)| s.clone()).collect();

            let qot_req = get_basic_qot::Request {
                c2s: get_basic_qot::C2s {
                    security_list: sec_list,
                },
            };

            let qot_resp: get_basic_qot::Response = client
                .request(PROTO_GET_BASIC_QOT, &qot_req.encode_to_vec())
                .await?;

            if qot_resp.ret_type != 0 {
                tracing::warn!("GetBasicQot batch failed: {:?}", qot_resp.ret_msg);
                continue;
            }

            if let Some(s2c) = qot_resp.s2c {
                for qot in &s2c.basic_qot_list {
                    // Find matching static info
                    let matching = chunk.iter().find(|(s, _, _, _)| {
                        s.market == qot.security.market && s.code == qot.security.code
                    });

                    if let Some((_, strike, expiry_str, opt_type)) = matching {
                        let opt_data = qot.option_ex_data.as_ref();
                        let iv = opt_data.map(|d| d.implied_volatility / 100.0).unwrap_or(0.0);
                        let oi = opt_data.map(|d| d.open_interest as u64).unwrap_or(0);

                        if iv <= 0.0 || oi == 0 {
                            continue;
                        }

                        let expiry =
                            chrono::NaiveDate::parse_from_str(expiry_str, "%Y-%m-%d").ok();
                        if let Some(expiry) = expiry {
                            contracts.push(OptionContract {
                                strike: *strike,
                                expiry,
                                option_type: *opt_type,
                                open_interest: oi,
                                implied_volatility: iv,
                                bid: qot.cur_price * 0.99, // approximate: cur_price ± 1%
                                ask: qot.cur_price * 1.01,
                            });
                        }
                    }
                }
            }
        }

        // Apply NextN filter if needed
        if let Some(ExpiryFilter::NextN(n)) = &expiry_filter {
            let mut expiries: Vec<chrono::NaiveDate> =
                contracts.iter().map(|c| c.expiry).collect();
            expiries.sort();
            expiries.dedup();
            let keep: std::collections::HashSet<_> =
                expiries.into_iter().take(*n as usize).collect();
            contracts.retain(|c| keep.contains(&c.expiry));
        }

        Ok(contracts)
    }

    fn name(&self) -> &'static str {
        "futu"
    }

    fn has_realtime(&self) -> bool {
        true
    }

    fn refresh_interval(&self) -> Duration {
        Duration::from_secs(5)
    }
}

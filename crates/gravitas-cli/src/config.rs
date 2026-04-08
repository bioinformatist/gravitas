use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub tradier_token: Option<String>,
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub futu_host: Option<String>,
    pub futu_port: Option<u16>,
}

impl Config {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gravitas")
            .join("config.toml")
    }
}

/// Resolved data source.
///
/// Auto-detection priority: Futu > Tradier > API
pub enum ResolvedSource {
    Futu { host: String, port: u16 },
    Direct { tradier_token: String },
    Api { api_key: String, api_base: String },
}

pub fn resolve_source(
    force_source: Option<&str>,
    config: &Config,
) -> Result<ResolvedSource, String> {
    let futu_host = std::env::var("FUTU_OPEND_HOST")
        .ok()
        .or_else(|| config.futu_host.clone());

    let futu_port = std::env::var("FUTU_OPEND_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .or(config.futu_port);

    let tradier_token = std::env::var("TRADIER_TOKEN")
        .ok()
        .or_else(|| config.tradier_token.clone());

    let api_key = std::env::var("GRAVITAS_API_KEY")
        .ok()
        .or_else(|| config.api_key.clone());

    let api_base = std::env::var("GRAVITAS_API_BASE")
        .ok()
        .or_else(|| config.api_base.clone())
        .unwrap_or_else(|| "http://localhost:8000".to_string());

    match force_source {
        Some("futu") => {
            let host = futu_host.ok_or("--source futu requires FUTU_OPEND_HOST")?;
            Ok(ResolvedSource::Futu {
                host,
                port: futu_port.unwrap_or(11111),
            })
        }
        Some("direct") => {
            let token = tradier_token.ok_or("--source direct requires TRADIER_TOKEN")?;
            Ok(ResolvedSource::Direct { tradier_token: token })
        }
        Some("api") => {
            let key = api_key.ok_or("--source api requires GRAVITAS_API_KEY")?;
            Ok(ResolvedSource::Api { api_key: key, api_base })
        }
        Some(other) => Err(format!("unknown source: {other}, expected 'futu', 'direct', or 'api'")),
        None => {
            // Auto-detect: Futu (realtime) > Tradier (15min delay) > API
            if let Some(host) = futu_host {
                Ok(ResolvedSource::Futu {
                    host,
                    port: futu_port.unwrap_or(11111),
                })
            } else if let Some(token) = tradier_token {
                Ok(ResolvedSource::Direct { tradier_token: token })
            } else if let Some(key) = api_key {
                Ok(ResolvedSource::Api { api_key: key, api_base })
            } else {
                Err(
                    "No credentials found. Options (in priority order):\n\
                     1. FUTU_OPEND_HOST + FUTU_OPEND_PORT (realtime, requires OpenD)\n\
                     2. TRADIER_TOKEN (15-min delay)\n\
                     3. GRAVITAS_API_KEY (via Shuttle API)\n\
                     Or create ~/.config/gravitas/config.toml"
                        .to_string(),
                )
            }
        }
    }
}

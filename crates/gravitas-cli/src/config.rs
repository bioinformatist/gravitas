use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub tradier_token: Option<String>,
    pub api_key: Option<String>,
    pub api_base: Option<String>,
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

/// Resolve the data source mode from CLI flags, env vars, and config file.
/// Priority: CLI flag > env var > config file.
/// Returns (mode, token/key).
pub enum ResolvedSource {
    Direct { tradier_token: String },
    Api { api_key: String, api_base: String },
}

pub fn resolve_source(
    force_source: Option<&str>,
    config: &Config,
) -> Result<ResolvedSource, String> {
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
        Some("direct") => {
            let token = tradier_token.ok_or("--source direct requires TRADIER_TOKEN env var or tradier_token in config")?;
            Ok(ResolvedSource::Direct { tradier_token: token })
        }
        Some("api") => {
            let key = api_key.ok_or("--source api requires GRAVITAS_API_KEY env var or api_key in config")?;
            Ok(ResolvedSource::Api { api_key: key, api_base })
        }
        Some(other) => Err(format!("unknown source: {other}, expected 'direct' or 'api'")),
        None => {
            // Auto-detect: prefer direct if tradier_token is available
            if let Some(token) = tradier_token {
                Ok(ResolvedSource::Direct { tradier_token: token })
            } else if let Some(key) = api_key {
                Ok(ResolvedSource::Api { api_key: key, api_base })
            } else {
                Err(
                    "No credentials found. Set TRADIER_TOKEN (for direct mode) or GRAVITAS_API_KEY (for API mode).\n\
                     Or create ~/.config/gravitas/config.toml with tradier_token or api_key."
                        .to_string(),
                )
            }
        }
    }
}

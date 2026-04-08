use axum::{extract::Path, extract::Query, extract::State, http::StatusCode, Json};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;

use gravitas_core::gex::compute_gex;
use gravitas_core::types::{ExpiryFilter, GexResult};

use crate::state::{AppState, CacheEntry};

#[derive(Deserialize)]
pub struct GexQuery {
    pub scenario: Option<f64>,
}

pub async fn get_gex(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<GexQuery>,
) -> Result<Json<GexResult>, StatusCode> {
    let symbol = symbol.to_uppercase();
    let cache_key = format!("{symbol}:{}", query.scenario.unwrap_or(0.0));

    // Check cache (skip if scenario is custom)
    if query.scenario.is_none() {
        if let Some(entry) = state.gex_cache.get(&cache_key) {
            if entry.inserted_at.elapsed() < state.cache_ttl {
                return Ok(Json(entry.data.clone()));
            }
        }
    }

    // Fetch options chain (may hit its own cache)
    let contracts = state
        .source
        .fetch_options_chain(&symbol, Some(ExpiryFilter::NextN(4)))
        .await
        .map_err(|e| {
            tracing::error!("fetch_options_chain error: {e}");
            StatusCode::BAD_GATEWAY
        })?;

    let mut spot = state.source.fetch_spot_price(&symbol).await.map_err(|e| {
        tracing::error!("fetch_spot_price error: {e}");
        StatusCode::BAD_GATEWAY
    })?;

    if let Some(pct) = query.scenario {
        spot *= 1.0 + pct / 100.0;
    }

    let result = compute_gex(&symbol, &contracts, spot, 0.05);

    if query.scenario.is_none() {
        state.gex_cache.insert(
            cache_key,
            CacheEntry {
                data: result.clone(),
                inserted_at: Instant::now(),
            },
        );
    }

    Ok(Json(result))
}

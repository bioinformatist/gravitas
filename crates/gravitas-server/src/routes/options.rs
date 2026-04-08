use axum::{extract::Path, extract::State, http::StatusCode, Json};
use std::sync::Arc;
use std::time::Instant;

use gravitas_core::types::{ExpiryFilter, OptionContract};

use crate::state::{AppState, CacheEntry};

pub async fn get_options(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<Json<Vec<OptionContract>>, StatusCode> {
    let symbol = symbol.to_uppercase();
    let cache_key = symbol.clone();

    // Check cache
    if let Some(entry) = state.options_cache.get(&cache_key) {
        if entry.inserted_at.elapsed() < state.cache_ttl {
            return Ok(Json(entry.data.clone()));
        }
    }

    let contracts = state
        .source
        .fetch_options_chain(&symbol, Some(ExpiryFilter::NextN(4)))
        .await
        .map_err(|e| {
            tracing::error!("fetch_options_chain error: {e}");
            StatusCode::BAD_GATEWAY
        })?;

    state.options_cache.insert(
        cache_key,
        CacheEntry {
            data: contracts.clone(),
            inserted_at: Instant::now(),
        },
    );

    Ok(Json(contracts))
}

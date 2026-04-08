use gravitas_fetch::source::DataSource;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gravitas_core::types::{GexResult, OptionContract};

pub struct CacheEntry<T> {
    pub data: T,
    pub inserted_at: Instant,
}

pub struct AppState {
    pub source: Box<dyn DataSource>,
    pub options_cache: DashMap<String, CacheEntry<Vec<OptionContract>>>,
    pub gex_cache: DashMap<String, CacheEntry<GexResult>>,
    pub cache_ttl: Duration,
}

impl AppState {
    pub fn new(source: Box<dyn DataSource>) -> Arc<Self> {
        let ttl = source.refresh_interval();
        Arc::new(Self {
            source,
            options_cache: DashMap::new(),
            gex_cache: DashMap::new(),
            cache_ttl: ttl,
        })
    }
}

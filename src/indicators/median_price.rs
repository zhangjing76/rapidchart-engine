use crate::indicators::derived::{hl2_store, latest_hl2};
use crate::NodeCache;
use crate::{CandleStore, RcSeries};

/// Median Price = (High + Low) / 2 — delegates to the shared `hl2_store` cache.
pub fn median_price_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    hl2_store(store, nodes)
}

pub fn latest_median_price_store(store: &CandleStore) -> Option<f64> {
    latest_hl2(store)
}
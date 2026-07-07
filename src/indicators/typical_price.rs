use crate::indicators::derived::{hlc3_store, latest_hlc3};
use crate::NodeCache;
use crate::{CandleStore, RcSeries};

/// Typical Price = (High + Low + Close) / 3 — delegates to the shared `hlc3_store` cache.
pub fn typical_price_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    hlc3_store(store, nodes)
}

pub fn latest_typical_price_store(store: &CandleStore) -> Option<f64> {
    latest_hlc3(store)
}

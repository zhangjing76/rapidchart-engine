use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Market Facilitation Index: (high - low) / volume
/// Measures price movement efficiency relative to volume.
pub fn market_facilitation_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "mfi_bw:hlv".to_string();
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        if store.volume[i] > 0.0 {
            out[i] = (store.high[i] - store.low[i]) / store.volume[i];
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_market_facilitation_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 { return None; }
    let i = store.len() - 1;
    if store.volume[i] > 0.0 { Some((store.high[i] - store.low[i]) / store.volume[i]) } else { None }
}
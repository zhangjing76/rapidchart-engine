use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// High Minus Low: simple H - L for each bar.
pub fn high_minus_low_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "hml:hl".to_string();
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let out: Vec<f64> = store.high.iter().zip(store.low.iter()).map(|(h, l)| h - l).collect();
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn latest_high_minus_low_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 { return None; }
    let i = store.len() - 1;
    Some(store.high[i] - store.low[i])
}
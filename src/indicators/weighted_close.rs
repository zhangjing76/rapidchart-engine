use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Weighted Close = (High + Low + 2*Close) / 4

pub fn weighted_close_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "weighted_close:hlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .zip(store.close.iter())
        .map(|((h, l), c)| (h + l + 2.0 * c) / 4.0)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_weighted_close_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 {
        return None;
    }
    let i = store.len() - 1;
    Some((store.high[i] + store.low[i] + 2.0 * store.close[i]) / 4.0)
}

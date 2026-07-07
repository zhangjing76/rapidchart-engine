use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Typical Price = (High + Low + Close) / 3


pub fn typical_price_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "typical_price:hlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .zip(store.close.iter())
        .map(|((h, l), c)| (h + l + c) / 3.0)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_typical_price_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 {
        return None;
    }
    let i = store.len() - 1;
    Some((store.high[i] + store.low[i] + store.close[i]) / 3.0)
}
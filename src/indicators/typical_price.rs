use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

/// Typical Price = (High + Low + Close) / 3
pub fn typical_price_series(bars: &[Bar]) -> Series {
    bars.iter()
        .map(|b| (b.high + b.low + b.close) / 3.0)
        .collect()
}

pub fn typical_price_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "typical_price:hlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = typical_price_series(bars);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

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

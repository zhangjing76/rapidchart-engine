use crate::{CandleStore, NodeCache, RcSeries};
use std::rc::Rc;

/// Cached (high + low) / 2 series.
#[allow(dead_code)]
pub fn hl2_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "hl2";
    if let Some(values) = nodes.get(key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(&h, &l)| (h + l) / 2.0)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key.to_string(), Rc::clone(&rc));
    rc
}

/// Latest (high + low) / 2 value.
pub fn latest_hl2(store: &CandleStore) -> Option<f64> {
    Some((*store.high.last()? + *store.low.last()?) / 2.0)
}

/// Cached (high + low + close) / 3 series.
pub fn hlc3_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "hlc3";
    if let Some(values) = nodes.get(key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .zip(store.close.iter())
        .map(|((&h, &l), &c)| (h + l + c) / 3.0)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key.to_string(), Rc::clone(&rc));
    rc
}

/// Latest (high + low + close) / 3 value.
pub fn latest_hlc3(store: &CandleStore) -> Option<f64> {
    Some((*store.high.last()? + *store.low.last()? + *store.close.last()?) / 3.0)
}

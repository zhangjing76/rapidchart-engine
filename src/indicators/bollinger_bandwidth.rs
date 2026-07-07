use crate::indicators::bollinger::bollinger_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Bollinger Bandwidth: (upper - lower) / middle * 100
const UPPER_SLOT: usize = 0;
const MIDDLE_SLOT: usize = 1;
const LOWER_SLOT: usize = 2;

pub fn bollinger_bandwidth_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("bb_bw:{}:{}", period, multiplier);
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let bb = bollinger_store(store, period, multiplier, nodes);
    let upper = &bb[UPPER_SLOT].values;
    let middle = &bb[MIDDLE_SLOT].values;
    let lower = &bb[LOWER_SLOT].values;
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        if !upper[i].is_nan()
            && !middle[i].is_nan()
            && !lower[i].is_nan()
            && middle[i].abs() > 1e-10
        {
            out[i] = ((upper[i] - lower[i]) / middle[i]) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_bollinger_bandwidth_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> Option<f64> {
    bollinger_bandwidth_store(store, period, multiplier, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

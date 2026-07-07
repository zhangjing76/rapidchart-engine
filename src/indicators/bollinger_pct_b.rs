use crate::indicators::bollinger::bollinger_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Bollinger %b: (close - lower) / (upper - lower)
/// Values above 1 = above upper band, below 0 = below lower band.
const UPPER_SLOT: usize = 0;
const LOWER_SLOT: usize = 2;

pub fn bollinger_pct_b_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("bb_pctb:{}:{}", period, multiplier);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let bb = bollinger_store(store, period, multiplier, nodes);
    let upper = &bb[UPPER_SLOT].values;
    let lower = &bb[LOWER_SLOT].values;
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let u = upper[i];
        let l = lower[i];
        if !u.is_nan() && !l.is_nan() && (u - l).abs() > 1e-10 {
            out[i] = (store.close[i] - l) / (u - l);
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_bollinger_pct_b_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> Option<f64> {
    bollinger_pct_b_store(store, period, multiplier, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

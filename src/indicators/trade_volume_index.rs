use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Trade Volume Index (TVI):
/// Cumulative volume based on tick direction.
/// If close > prev_close: add volume
/// If close < prev_close: subtract volume
/// If close == prev_close: no change
pub fn trade_volume_index_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "tvi:cv".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len == 0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    out[0] = 0.0;
    for i in 1..len {
        let diff = store.close[i] - store.close[i - 1];
        if diff > 0.0 {
            out[i] = out[i - 1] + store.volume[i];
        } else if diff < 0.0 {
            out[i] = out[i - 1] - store.volume[i];
        } else {
            out[i] = out[i - 1];
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_trade_volume_index_store(store: &CandleStore, prev: Option<&[f64]>) -> Option<f64> {
    let len = store.len();
    if len == 0 {
        return None;
    }
    if len == 1 {
        return Some(0.0);
    }
    let prev_tvi = prev
        .and_then(|s| s.get(len - 2).copied())
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
        .unwrap_or(0.0);
    let diff = store.close[len - 1] - store.close[len - 2];
    if diff > 0.0 {
        Some(prev_tvi + store.volume[len - 1])
    } else if diff < 0.0 {
        Some(prev_tvi - store.volume[len - 1])
    } else {
        Some(prev_tvi)
    }
}

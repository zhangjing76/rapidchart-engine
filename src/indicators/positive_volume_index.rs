use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Positive Volume Index (PVI):
/// Starts at 1000. Only changes on days when volume increases.
/// PVI = prev_PVI + (close - prev_close) / prev_close * prev_PVI  (when volume > prev_volume)
pub fn positive_volume_index_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "pvi:cv".to_string();
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len == 0 { let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); return rc; }
    out[0] = 1000.0;
    for i in 1..len {
        if store.volume[i] > store.volume[i-1] && store.close[i-1] != 0.0 {
            let roc = (store.close[i] - store.close[i-1]) / store.close[i-1];
            out[i] = out[i-1] + roc * out[i-1];
        } else {
            out[i] = out[i-1];
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_positive_volume_index_store(store: &CandleStore, prev: Option<&[f64]>) -> Option<f64> {
    let len = store.len();
    if len == 0 { return None; }
    if len == 1 { return Some(1000.0); }
    let prev_pvi = prev.and_then(|s| s.get(len - 2).copied())
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
        .unwrap_or(1000.0);
    if store.volume[len-1] > store.volume[len-2] && store.close[len-2] != 0.0 {
        let roc = (store.close[len-1] - store.close[len-2]) / store.close[len-2];
        Some(prev_pvi + roc * prev_pvi)
    } else {
        Some(prev_pvi)
    }
}
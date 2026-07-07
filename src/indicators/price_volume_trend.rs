use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Price Volume Trend (PVT):
/// Cumulative sum of volume * ((close - prev_close) / prev_close)
pub fn price_volume_trend_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "pvt:cv".to_string();
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
        let prev = out[i - 1];
        if store.close[i - 1] != 0.0 {
            let roc = (store.close[i] - store.close[i - 1]) / store.close[i - 1];
            out[i] = prev + store.volume[i] * roc;
        } else {
            out[i] = prev;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_price_volume_trend_store(store: &CandleStore, prev: Option<&[f64]>) -> Option<f64> {
    let len = store.len();
    if len == 0 {
        return None;
    }
    if len == 1 {
        return Some(0.0);
    }
    let prev_pvt = prev
        .and_then(|s| s.get(len - 2).copied())
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
        .unwrap_or(0.0);
    if store.close[len - 2] != 0.0 {
        let roc = (store.close[len - 1] - store.close[len - 2]) / store.close[len - 2];
        Some(prev_pvt + store.volume[len - 1] * roc)
    } else {
        Some(prev_pvt)
    }
}

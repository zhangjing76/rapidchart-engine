use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// True Range: max(H-L, |H-PC|, |L-PC|) per bar. First bar = H-L.
pub fn true_range_series_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "true_range:hlc".to_string();
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len == 0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    out[0] = store.high[0] - store.low[0];
    for i in 1..len {
        out[i] = (store.high[i] - store.low[i])
            .max((store.high[i] - store.close[i - 1]).abs())
            .max((store.low[i] - store.close[i - 1]).abs());
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_true_range_store(store: &CandleStore) -> Option<f64> {
    let len = store.len();
    if len == 0 {
        return None;
    }
    if len == 1 {
        return Some(store.high[0] - store.low[0]);
    }
    let i = len - 1;
    Some(
        (store.high[i] - store.low[i])
            .max((store.high[i] - store.close[i - 1]).abs())
            .max((store.low[i] - store.close[i - 1]).abs()),
    )
}

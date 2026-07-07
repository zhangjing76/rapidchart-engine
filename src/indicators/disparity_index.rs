use crate::indicators::ema::ema_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Disparity Index: ((close - EMA(close, period)) / EMA(close, period)) * 100
/// Measures percentage distance of close from its moving average.
pub fn disparity_index_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("disparity:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema = ema_close_store(store, period, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let e = ema[i];
        if !e.is_nan() && e.abs() > 1e-10 {
            out[i] = ((store.close[i] - e) / e) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_disparity_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    disparity_index_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

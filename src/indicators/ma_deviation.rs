use crate::indicators::sma::sma_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Moving Average Deviation: ((close - SMA) / SMA) * 100
pub fn ma_deviation_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ma_dev:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma = sma_close_store(store, period, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let s = sma[i];
        if !s.is_nan() && s.abs() > 1e-10 {
            out[i] = ((store.close[i] - s) / s) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_ma_deviation_store(store: &CandleStore, period: usize) -> Option<f64> {
    ma_deviation_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

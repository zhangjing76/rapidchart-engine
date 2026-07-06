use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use crate::indicators::sma::sma_close_store;
use std::collections::HashMap;
use std::rc::Rc;

/// RAVI (Range Action Verification Index):
/// |SMA(short) - SMA(long)| / SMA(long) * 100
pub fn ravi_store(store: &CandleStore, short: usize, long: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ravi:close:{}:{}", short, long);
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let sma_short = sma_close_store(store, short, nodes);
    let sma_long = sma_close_store(store, long, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let s = sma_short[i];
        let l = sma_long[i];
        if !s.is_nan() && !l.is_nan() && l.abs() > 1e-10 {
            out[i] = ((s - l).abs() / l) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_ravi_store(store: &CandleStore, short: usize, long: usize) -> Option<f64> {
    ravi_store(store, short, long, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}
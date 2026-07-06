use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Vertical Horizontal Filter:
/// VHF = |close - close[period]| / SUM(|close[i] - close[i-1]|, period)
/// High VHF = trending, Low VHF = ranging.
pub fn vertical_horizontal_filter_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("vhf:close:{period}");
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period + 1 {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); return rc;
    }
    for i in period..len {
        let numerator = (store.close[i] - store.close[i - period]).abs();
        let denominator: f64 = (i + 1 - period..=i).map(|j| (store.close[j] - store.close[j - 1]).abs()).sum();
        if denominator > 1e-10 { out[i] = numerator / denominator; }
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn latest_vertical_horizontal_filter_store(store: &CandleStore, period: usize) -> Option<f64> {
    vertical_horizontal_filter_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// QStick: SMA of (close - open) over period.
/// Positive = bullish candle bodies dominate, Negative = bearish.
pub fn qstick_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("qstick:oc:{period}");
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); return rc;
    }
    for i in period - 1..len {
        let sum: f64 = (i + 1 - period..=i).map(|j| store.close[j] - store.open[j]).sum();
        out[i] = sum / period as f64;
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn latest_qstick_store(store: &CandleStore, period: usize) -> Option<f64> {
    qstick_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}
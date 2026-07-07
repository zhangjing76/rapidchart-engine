use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Ulcer Index: measures downside volatility/drawdown severity.
/// UI = sqrt(SUM(((close - max_close_over_period) / max_close_over_period * 100)^2, period) / period)
pub fn ulcer_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ulcer:close:{period}");
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..len {
        let max_close = store.close[i + 1 - period..=i]
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        if max_close.abs() < 1e-10 {
            continue;
        }
        let sum_sq: f64 = store.close[i + 1 - period..=i]
            .iter()
            .map(|&c| {
                let pct_drawdown = ((c - max_close) / max_close) * 100.0;
                pct_drawdown * pct_drawdown
            })
            .sum();
        out[i] = (sum_sq / period as f64).sqrt();
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_ulcer_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    ulcer_index_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

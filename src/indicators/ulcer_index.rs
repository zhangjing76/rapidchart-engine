use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Ulcer Index: measures downside volatility/drawdown severity.
/// UI = sqrt(SUM(((close - max_close_over_period) / max_close_over_period * 100)^2, period) / period)
pub fn ulcer_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ulcer:close:{period}");
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); return rc;
    }
    for i in period - 1..len {
        let max_close = store.close[i + 1 - period..=i].iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        if max_close.abs() < 1e-10 { continue; }
        let sum_sq: f64 = store.close[i + 1 - period..=i].iter().map(|&c| {
            let pct_drawdown = ((c - max_close) / max_close) * 100.0;
            pct_drawdown * pct_drawdown
        }).sum();
        out[i] = (sum_sq / period as f64).sqrt();
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn ulcer_index_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("ulcer:close:{period}");
    if let Some(v) = nodes.get(&key) { return (**v).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period { nodes.insert(key, Rc::new(out.clone())); return out; }
    for i in period - 1..len {
        let max_close = bars[i + 1 - period..=i].iter().map(|b| b.close).fold(f64::NEG_INFINITY, f64::max);
        if max_close.abs() < 1e-10 { continue; }
        let sum_sq: f64 = bars[i + 1 - period..=i].iter().map(|b| {
            let pct = ((b.close - max_close) / max_close) * 100.0;
            pct * pct
        }).sum();
        out[i] = (sum_sq / period as f64).sqrt();
    }
    nodes.insert(key, Rc::new(out.clone())); out
}
pub fn latest_ulcer_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    ulcer_index_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

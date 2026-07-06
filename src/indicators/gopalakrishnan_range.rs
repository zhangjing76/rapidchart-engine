use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Gopalakrishnan Range Index: log(highest - lowest over period) / log(period)
pub fn gopalakrishnan_range_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("gapo:hl:{period}");
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); return rc;
    }
    let log_period = (period as f64).ln();
    for i in period - 1..len {
        let hh = store.high[i+1-period..=i].iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = store.low[i+1-period..=i].iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = hh - ll;
        if range > 0.0 && log_period > 0.0 { out[i] = range.ln() / log_period; }
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn gopalakrishnan_range_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("gapo:hl:{period}");
    if let Some(v) = nodes.get(&key) { return (**v).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period { nodes.insert(key, Rc::new(out.clone())); return out; }
    let log_period = (period as f64).ln();
    for i in period - 1..len {
        let hh = bars[i+1-period..=i].iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let ll = bars[i+1-period..=i].iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let range = hh - ll;
        if range > 0.0 && log_period > 0.0 { out[i] = range.ln() / log_period; }
    }
    nodes.insert(key, Rc::new(out.clone())); out
}
pub fn latest_gopalakrishnan_range_store(store: &CandleStore, period: usize) -> Option<f64> {
    gopalakrishnan_range_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

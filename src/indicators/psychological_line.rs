use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Psychological Line: percentage of bars that closed up over the period.
/// PSY = (bars where close > prev_close) / period * 100
pub fn psychological_line_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("psy:close:{period}");
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period + 1 {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); return rc;
    }
    for i in period..len {
        let up = (i + 1 - period..=i).filter(|&j| store.close[j] > store.close[j - 1]).count();
        out[i] = (up as f64 / period as f64) * 100.0;
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn psychological_line_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("psy:close:{period}");
    if let Some(v) = nodes.get(&key) { return (**v).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period + 1 { nodes.insert(key, Rc::new(out.clone())); return out; }
    for i in period..len {
        let up = (i + 1 - period..=i).filter(|&j| bars[j].close > bars[j - 1].close).count();
        out[i] = (up as f64 / period as f64) * 100.0;
    }
    nodes.insert(key, Rc::new(out.clone())); out
}
pub fn latest_psychological_line_store(store: &CandleStore, period: usize) -> Option<f64> {
    psychological_line_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

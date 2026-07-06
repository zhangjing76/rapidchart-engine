use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Volume Rate of Change: (volume - volume[period]) / volume[period] * 100
pub fn volume_roc_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("vol_roc:v:{period}");
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len <= period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); return rc;
    }
    for i in period..len {
        let prev = store.volume[i - period];
        if prev > 0.0 {
            out[i] = ((store.volume[i] - prev) / prev) * 100.0;
        }
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn volume_roc_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("vol_roc:v:{period}");
    if let Some(v) = nodes.get(&key) { return (**v).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len <= period { nodes.insert(key, Rc::new(out.clone())); return out; }
    for i in period..len {
        let prev = bars[i - period].volume;
        if prev > 0.0 { out[i] = ((bars[i].volume - prev) / prev) * 100.0; }
    }
    nodes.insert(key, Rc::new(out.clone())); out
}
pub fn latest_volume_roc_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() <= period { return None; }
    let i = store.len() - 1;
    let prev = store.volume[i - period];
    if prev > 0.0 { Some(((store.volume[i] - prev) / prev) * 100.0) } else { None }
}

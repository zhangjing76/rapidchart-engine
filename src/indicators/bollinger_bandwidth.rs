use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use crate::indicators::bollinger::bollinger_store;
use std::collections::HashMap;
use std::rc::Rc;

/// Bollinger Bandwidth: (upper - lower) / middle * 100
pub fn bollinger_bandwidth_store(store: &CandleStore, period: usize, multiplier: f64, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("bb_bw:{}:{}", period, multiplier);
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let bb = bollinger_store(store, period, multiplier, nodes);
    let upper = bb.iter().find(|o| o.name == "upper").map(|o| &o.values);
    let middle = bb.iter().find(|o| o.name == "middle").map(|o| &o.values);
    let lower = bb.iter().find(|o| o.name == "lower").map(|o| &o.values);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if let (Some(u), Some(m), Some(l)) = (upper, middle, lower) {
        for i in 0..len {
            if !u[i].is_nan() && !m[i].is_nan() && !l[i].is_nan() && m[i].abs() > 1e-10 {
                out[i] = ((u[i] - l[i]) / m[i]) * 100.0;
            }
        }
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn bollinger_bandwidth_node(bars: &[Bar], period: usize, multiplier: f64, nodes: &mut NodeCache) -> Series {
    let key = format!("bb_bw:{}:{}", period, multiplier);
    if let Some(v) = nodes.get(&key) { return (**v).clone(); }
    let bb = crate::indicators::bollinger::bollinger(bars, period, multiplier, nodes);
    let upper = bb.iter().find(|o| o.name == "upper").map(|o| &o.values);
    let middle = bb.iter().find(|o| o.name == "middle").map(|o| &o.values);
    let lower = bb.iter().find(|o| o.name == "lower").map(|o| &o.values);
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if let (Some(u), Some(m), Some(l)) = (upper, middle, lower) {
        for i in 0..len {
            if !u[i].is_nan() && !m[i].is_nan() && !l[i].is_nan() && m[i].abs() > 1e-10 {
                out[i] = ((u[i] - l[i]) / m[i]) * 100.0;
            }
        }
    }
    nodes.insert(key, Rc::new(out.clone())); out
}
pub fn latest_bollinger_bandwidth_store(store: &CandleStore, period: usize, multiplier: f64) -> Option<f64> {
    bollinger_bandwidth_store(store, period, multiplier, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

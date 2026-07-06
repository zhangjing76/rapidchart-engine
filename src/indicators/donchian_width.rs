use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use crate::indicators::donchian::donchian_store;
use std::collections::HashMap;
use std::rc::Rc;

/// Donchian Width: (upper - lower) / middle * 100
pub fn donchian_width_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("donchian_width:{}", period);
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let dc = donchian_store(store, period, nodes);
    let upper = dc.iter().find(|o| o.name == "upper").map(|o| &o.values);
    let middle = dc.iter().find(|o| o.name == "middle").map(|o| &o.values);
    let lower = dc.iter().find(|o| o.name == "lower").map(|o| &o.values);
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
pub fn latest_donchian_width_store(store: &CandleStore, period: usize) -> Option<f64> {
    donchian_width_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}
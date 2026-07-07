use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use crate::indicators::donchian::donchian_store;
use std::collections::HashMap;
use std::rc::Rc;

/// Donchian Width: (upper - lower) / middle * 100
const UPPER_SLOT: usize = 0;
const MIDDLE_SLOT: usize = 1;
const LOWER_SLOT: usize = 2;

pub fn donchian_width_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("donchian_width:{}", period);
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let dc = donchian_store(store, period, nodes);
    let upper = &dc[UPPER_SLOT].values;
    let middle = &dc[MIDDLE_SLOT].values;
    let lower = &dc[LOWER_SLOT].values;
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        if !upper[i].is_nan()
            && !middle[i].is_nan()
            && !lower[i].is_nan()
            && middle[i].abs() > 1e-10
        {
            out[i] = ((upper[i] - lower[i]) / middle[i]) * 100.0;
        }
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn latest_donchian_width_store(store: &CandleStore, period: usize) -> Option<f64> {
    donchian_width_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

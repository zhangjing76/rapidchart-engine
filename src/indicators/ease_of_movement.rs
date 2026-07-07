use crate::indicators::derived::hl2_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Ease of Movement (EMV):
/// Distance Moved = ((high + low)/2 - (prev_high + prev_low)/2)
/// Box Ratio = volume / (high - low)
/// EMV = Distance Moved / Box Ratio (then smoothed with SMA of period)
pub fn ease_of_movement_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("emv:hlv:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let hl2 = hl2_store(store, nodes);
    let mut raw = vec![f64::NAN; len];
    for i in 1..len {
        let distance = hl2[i] - hl2[i - 1];
        let hl_diff = store.high[i] - store.low[i];
        if hl_diff.abs() > 1e-10 && store.volume[i] > 0.0 {
            let box_ratio = (store.volume[i] / 10000.0) / hl_diff;
            if box_ratio.abs() > 1e-10 {
                raw[i] = distance / box_ratio;
            }
        }
    }
    // SMA smoothing
    let mut out = vec![f64::NAN; len];
    if period > 0 && len >= period {
        for i in period..len {
            let window = &raw[i + 1 - period..=i];
            let valid: Vec<f64> = window.iter().filter(|v| !v.is_nan()).copied().collect();
            if !valid.is_empty() {
                out[i] = valid.iter().sum::<f64>() / valid.len() as f64;
            }
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_ease_of_movement_store(store: &CandleStore, period: usize) -> Option<f64> {
    ease_of_movement_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

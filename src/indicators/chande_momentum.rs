use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Chande Momentum Oscillator (CMO):
/// CMO = ((sum_up - sum_down) / (sum_up + sum_down)) * 100
/// where sum_up = sum of positive changes over period,
///       sum_down = sum of absolute negative changes over period.
pub fn chande_momentum_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("cmo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period + 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period..len {
        let mut sum_up = 0.0;
        let mut sum_down = 0.0;
        for j in (i + 1 - period)..=i {
            let diff = store.close[j] - store.close[j - 1];
            if diff > 0.0 {
                sum_up += diff;
            } else {
                sum_down += -diff;
            }
        }
        let total = sum_up + sum_down;
        if total > 0.0 {
            out[i] = ((sum_up - sum_down) / total) * 100.0;
        } else {
            out[i] = 0.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_chande_momentum_store(store: &CandleStore, period: usize) -> Option<f64> {
    chande_momentum_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

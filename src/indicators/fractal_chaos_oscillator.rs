use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Fractal Chaos Oscillator:
/// +1 when a fractal high is detected, -1 when a fractal low is detected, 0 otherwise.
/// A fractal high occurs when bar[mid].high is the highest among 5 bars.
/// A fractal low occurs when bar[mid].low is the lowest among 5 bars.
pub fn fractal_chaos_oscillator_store(store: &CandleStore, _nodes: &mut NodeCache) -> RcSeries {
    let key = "fco:hl".to_string();
    if let Some(values) = _nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 5 {
        let rc = Rc::new(out);
        _nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in 4..len {
        let mid = i - 2;
        let is_high = store.high[mid] > store.high[mid - 2]
            && store.high[mid] > store.high[mid - 1]
            && store.high[mid] > store.high[mid + 1]
            && store.high[mid] > store.high[mid + 2];
        let is_low = store.low[mid] < store.low[mid - 2]
            && store.low[mid] < store.low[mid - 1]
            && store.low[mid] < store.low[mid + 1]
            && store.low[mid] < store.low[mid + 2];
        if is_high {
            out[i] = 1.0;
        } else if is_low {
            out[i] = -1.0;
        } else {
            out[i] = 0.0;
        }
    }
    let rc = Rc::new(out);
    _nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_fractal_chaos_oscillator_store(store: &CandleStore) -> Option<f64> {
    fractal_chaos_oscillator_store(store, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

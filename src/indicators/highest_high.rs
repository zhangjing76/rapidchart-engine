use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries};
use std::rc::Rc;

/// Highest High Value over a rolling window of `period` bars.


pub fn highest_high_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("highest_high:h:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period > 0 && store.len() >= period {
        for i in period - 1..store.len() {
            let max = store.high[i + 1 - period..=i]
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            out[i] = max;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

#[allow(dead_code)]
pub fn latest_highest_high(bars: &[Bar], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    let max = bars[bars.len() - period..]
        .iter()
        .map(|b| b.high)
        .fold(f64::NEG_INFINITY, f64::max);
    Some(max)
}

pub fn latest_highest_high_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let max = store.high[start..]
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    Some(max)
}
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Lowest Low Value over a rolling window of `period` bars.


pub fn lowest_low_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("lowest_low:l:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period > 0 && store.len() >= period {
        for i in period - 1..store.len() {
            let min = store.low[i + 1 - period..=i]
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b));
            out[i] = min;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_lowest_low_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let min = store.low[start..]
        .iter()
        .fold(f64::INFINITY, |a, &b| a.min(b));
    Some(min)
}
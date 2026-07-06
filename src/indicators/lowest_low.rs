use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

/// Lowest Low Value over a rolling window of `period` bars.
pub fn lowest_low(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.is_empty() {
        return out;
    }
    for i in period - 1..bars.len() {
        let min = bars[i + 1 - period..=i]
            .iter()
            .map(|b| b.low)
            .fold(f64::INFINITY, f64::min);
        out[i] = min;
    }
    out
}

pub fn lowest_low_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("lowest_low:l:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = lowest_low(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

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

#[allow(dead_code)]
pub fn latest_lowest_low(bars: &[Bar], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    let min = bars[bars.len() - period..]
        .iter()
        .map(|b| b.low)
        .fold(f64::INFINITY, f64::min);
    Some(min)
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

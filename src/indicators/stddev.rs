use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

pub fn stddev(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let mean = window.iter().map(|bar| bar.close).sum::<f64>() / period as f64;
        let variance = window
            .iter()
            .map(|bar| {
                let diff = bar.close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        out[index] = variance.sqrt();
    }
    out
}
pub fn stddev_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("stddev:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = stddev(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}
pub fn stddev_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("stddev:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for index in period - 1..store.len() {
        let window = &store.close[index + 1 - period..=index];
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window
            .iter()
            .map(|close| {
                let diff = close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        out[index] = variance.sqrt();
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_stddev(bars: &[Bar], period: usize) -> Option<f64> {
    stddev(bars, period).last().copied().and_then(nan_to_none)
}
pub fn latest_stddev_store(store: &CandleStore, period: usize) -> Option<f64> {
    stddev_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

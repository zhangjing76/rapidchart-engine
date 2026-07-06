use crate::indicators::wma::{wma_close, wma_from_values, wma_store};
use crate::nan_to_none;
use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

#[allow(dead_code)]
pub fn hma(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("hma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    if period == 0 {
        return vec![f64::NAN; bars.len()];
    }
    let half_period = (period / 2).max(1);
    let sqrt_period = ((period as f64).sqrt().round() as usize).max(1);
    let half = wma_close(bars, half_period, nodes);
    let full = wma_close(bars, period, nodes);
    let raw: Vec<_> = half
        .iter()
        .zip(full.iter())
        .map(|(half, full)| match (half, full) {
            (half, full) if !half.is_nan() && !full.is_nan() => 2.0 * *half - *full,
            _ => f64::NAN,
        })
        .collect();
    let values = wma_from_values(&raw, sqrt_period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}
pub fn hma_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("hma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    if period == 0 {
        return Rc::new(vec![f64::NAN; store.len()]);
    }
    let half_period = (period / 2).max(1);
    let sqrt_period = ((period as f64).sqrt().round() as usize).max(1);
    let half = wma_store(store, half_period, nodes);
    let full = wma_store(store, period, nodes);
    let raw: Vec<_> = half
        .iter()
        .zip(full.iter())
        .map(|(half, full)| match (half, full) {
            (half, full) if !half.is_nan() && !full.is_nan() => 2.0 * *half - *full,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(wma_from_values(&raw, sqrt_period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}
#[allow(dead_code)]
pub fn latest_hma(bars: &[Bar], period: usize) -> Option<f64> {
    hma(bars, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}
pub fn latest_hma_store(store: &CandleStore, period: usize) -> Option<f64> {
    hma_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}
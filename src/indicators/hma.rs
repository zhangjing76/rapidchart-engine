use crate::indicators::wma::{wma_from_values, wma_store};
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

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
pub fn latest_hma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 {
        return None;
    }
    let half_period = (period / 2).max(1);
    let sqrt_period = ((period as f64).sqrt().round() as usize).max(1);
    // We need enough bars for the full pipeline
    // WMA(period) needs `period` bars, then the diff series needs `sqrt_period` values
    let needed = period + sqrt_period - 1;
    if store.len() < needed {
        return None;
    }
    // Compute the raw series (2*WMA_half - WMA_full) for the last sqrt_period positions
    let mut raw = Vec::with_capacity(sqrt_period);
    for end in (store.len() - sqrt_period)..store.len() {
        let half_start = end + 1 - half_period;
        let full_start = end + 1 - period;
        let wma_half = wma_window(&store.close[half_start..=end], half_period);
        let wma_full = wma_window(&store.close[full_start..=end], period);
        match (wma_half, wma_full) {
            (Some(h), Some(f)) => raw.push(2.0 * h - f),
            _ => return None,
        }
    }
    wma_window(&raw, sqrt_period)
}

fn wma_window(values: &[f64], period: usize) -> Option<f64> {
    if values.len() < period || period == 0 {
        return None;
    }
    let window = &values[values.len() - period..];
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;
    for (i, &v) in window.iter().enumerate() {
        if v.is_nan() {
            return None;
        }
        let w = (i + 1) as f64;
        weighted_sum += v * w;
        weight_total += w;
    }
    Some(weighted_sum / weight_total)
}

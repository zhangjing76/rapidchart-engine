use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn wma_from_values(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    let denominator = (period * (period + 1) / 2) as f64;
    for index in period - 1..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        let weighted_sum = window
            .iter()
            .enumerate()
            .map(|(offset, value)| (offset + 1) as f64 * value)
            .sum::<f64>();
        out[index] = weighted_sum / denominator;
    }
    out
}
pub fn wma_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("wma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let values = store.close.to_vec();
    let rc = Rc::new(wma_from_values(&values, period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_wma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let denominator = (period * (period + 1) / 2) as f64;
    let start = store.len() - period;
    let weighted_sum = store.close[start..]
        .iter()
        .enumerate()
        .map(|(offset, value)| (offset + 1) as f64 * value)
        .sum::<f64>();
    Some(weighted_sum / denominator)
}

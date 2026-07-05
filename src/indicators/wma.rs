use crate::{Bar, CandleStore, Series, RcSeries};
use crate::NodeCache;
use crate::{rc_into_owned, nan_to_none};
use std::rc::Rc;

pub fn wma(bars: &[Bar], period: usize) -> Series {    let values: Vec<_> = bars.iter().map(|bar| bar.close).collect();    wma_from_values(&values, period)}
pub fn wma_close(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {    let key = format!("wma:close:{period}");    if let Some(values) = nodes.get(&key) {        return (**values).clone();    }    let values = wma(bars, period);    nodes.insert(key, Rc::new(values.clone()));    values}
pub fn wma_from_values(values: &[f64], period: usize) -> Series {    let mut out = vec![f64::NAN; values.len()];    if period == 0 || values.len() < period {        return out;    }    let denominator = (period * (period + 1) / 2) as f64;    for index in period - 1..values.len() {        let window = &values[index + 1 - period..=index];        if window.iter().any(|value| value.is_nan()) {            continue;        }        let weighted_sum = window            .iter()            .enumerate()            .map(|(offset, value)| (offset + 1) as f64 * value)            .sum::<f64>();        out[index] = weighted_sum / denominator;    }    out}
pub fn wma_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {    let key = format!("wma:close:{period}");    if let Some(values) = nodes.get(&key) {        return Rc::clone(values);    }    let values: Vec<_> = store.close.iter().copied().collect();    let rc = Rc::new(wma_from_values(&values, period));    nodes.insert(key, Rc::clone(&rc));    rc}
pub fn latest_wma(bars: &[Bar], period: usize) -> Option<f64> {    wma(bars, period).last().copied().and_then(nan_to_none)}
pub fn latest_wma_store(store: &CandleStore, period: usize) -> Option<f64> {    if period == 0 || store.len() < period {        return None;    }    let denominator = (period * (period + 1) / 2) as f64;    let start = store.len() - period;    let weighted_sum = store.close[start..]        .iter()        .enumerate()        .map(|(offset, value)| (offset + 1) as f64 * value)        .sum::<f64>();    Some(weighted_sum / denominator)}

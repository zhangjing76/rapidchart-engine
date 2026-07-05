use crate::nan_to_none;
use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

pub fn linear_regression(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }
    let n = period as f64;
    let sum_x = (0..period).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..period).map(|x| (x * x) as f64).sum::<f64>();
    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator == 0.0 {
        return out;
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let sum_y = window.iter().map(|bar| bar.close).sum::<f64>();
        let sum_xy = window
            .iter()
            .enumerate()
            .map(|(offset, bar)| offset as f64 * bar.close)
            .sum::<f64>();
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;
        out[index] = intercept + slope * (period - 1) as f64;
    }
    out
}
pub fn linear_regression_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("linreg:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = linear_regression(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}
pub fn linear_regression_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("linreg:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let n = period as f64;
    let sum_x = (0..period).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..period).map(|x| (x * x) as f64).sum::<f64>();
    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator == 0.0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
        let window = &store.close[index + 1 - period..=index];
        let sum_y = window.iter().sum::<f64>();
        let sum_xy = window
            .iter()
            .enumerate()
            .map(|(offset, close)| offset as f64 * close)
            .sum::<f64>();
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;
        *item = intercept + slope * (period - 1) as f64;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
#[allow(dead_code)]
pub fn latest_linear_regression(bars: &[Bar], period: usize) -> Option<f64> {
    linear_regression(bars, period)
        .last()
        .copied()
        .and_then(nan_to_none)
}
pub fn latest_linear_regression_store(store: &CandleStore, period: usize) -> Option<f64> {
    linear_regression_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

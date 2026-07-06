use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Correlation Coefficient (single-symbol): measures the rolling Pearson
/// correlation between price and a linear time axis over `period` bars.
/// This indicates how linearly trending the price is.
/// +1 = perfect uptrend, -1 = perfect downtrend, 0 = no linear trend.
pub fn correlation_coefficient_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("correl:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }

    let n = period as f64;
    let sum_x = (0..period).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..period).map(|x| (x * x) as f64).sum::<f64>();

    for i in period - 1..len {
        let window = &store.close[i + 1 - period..=i];
        let sum_y: f64 = window.iter().sum();
        let sum_yy: f64 = window.iter().map(|y| y * y).sum();
        let sum_xy: f64 = window
            .iter()
            .enumerate()
            .map(|(x, y)| x as f64 * y)
            .sum();

        let numerator = n * sum_xy - sum_x * sum_y;
        let denom_x = (n * sum_xx - sum_x * sum_x).sqrt();
        let denom_y = (n * sum_yy - sum_y * sum_y).sqrt();
        let denominator = denom_x * denom_y;

        if denominator > 0.0 {
            out[i] = numerator / denominator;
        }
    }

    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn correlation_coefficient_node(
    bars: &[Bar],
    period: usize,
    nodes: &mut NodeCache,
) -> Series {
    let key = format!("correl:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }

    let n = period as f64;
    let sum_x = (0..period).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..period).map(|x| (x * x) as f64).sum::<f64>();

    for i in period - 1..len {
        let window = &bars[i + 1 - period..=i];
        let sum_y: f64 = window.iter().map(|b| b.close).sum();
        let sum_yy: f64 = window.iter().map(|b| b.close * b.close).sum();
        let sum_xy: f64 = window
            .iter()
            .enumerate()
            .map(|(x, b)| x as f64 * b.close)
            .sum();

        let numerator = n * sum_xy - sum_x * sum_y;
        let denom_x = (n * sum_xx - sum_x * sum_x).sqrt();
        let denom_y = (n * sum_yy - sum_y * sum_y).sqrt();
        let denominator = denom_x * denom_y;

        if denominator > 0.0 {
            out[i] = numerator / denominator;
        }
    }

    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_correlation_coefficient_store(store: &CandleStore, period: usize) -> Option<f64> {
    correlation_coefficient_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

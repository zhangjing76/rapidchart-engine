use crate::NodeCache;
use crate::{CandleStore, RcSeries};
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
        let sum_xy: f64 = window.iter().enumerate().map(|(x, y)| x as f64 * y).sum();

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

pub fn latest_correlation_coefficient_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period < 2 || store.len() < period {
        return None;
    }
    let n = period as f64;
    let sum_x: f64 = (0..period).map(|x| x as f64).sum();
    let sum_xx: f64 = (0..period).map(|x| (x * x) as f64).sum();
    let window = &store.close[store.len() - period..];
    let sum_y: f64 = window.iter().sum();
    let sum_yy: f64 = window.iter().map(|y| y * y).sum();
    let sum_xy: f64 = window.iter().enumerate().map(|(x, y)| x as f64 * y).sum();
    let numerator = n * sum_xy - sum_x * sum_y;
    let denom_x = (n * sum_xx - sum_x * sum_x).sqrt();
    let denom_y = (n * sum_yy - sum_y * sum_y).sqrt();
    let denominator = denom_x * denom_y;
    if denominator > 0.0 {
        Some(numerator / denominator)
    } else {
        None
    }
}

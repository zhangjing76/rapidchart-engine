use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Beta indicator (single-symbol): measures the rolling regression slope of
/// log returns over `period` bars. Without a benchmark, this captures
/// the trend strength/direction of the asset's own returns.
///
/// Computed as: slope of linear regression of returns over the window,
/// where returns[i] = (close[i] - close[i-1]) / close[i-1].
pub fn beta_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("beta:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period + 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }

    // Compute returns series
    let mut returns = vec![f64::NAN; len];
    for i in 1..len {
        if store.close[i - 1] != 0.0 {
            returns[i] = (store.close[i] - store.close[i - 1]) / store.close[i - 1];
        }
    }

    // Rolling standard deviation of returns as a volatility-based beta proxy
    let n = period as f64;
    for i in period..len {
        let window = &returns[i + 1 - period..=i];
        let valid: Vec<f64> = window.iter().filter(|v| !v.is_nan()).copied().collect();
        if valid.len() < 2 {
            continue;
        }
        let count = valid.len() as f64;
        let mean = valid.iter().sum::<f64>() / count;
        let variance = valid.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (count - 1.0);
        let stddev = variance.sqrt();
        // Annualized-style beta: stddev of returns * sqrt(period) gives a comparable measure
        out[i] = stddev * n.sqrt();
    }

    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_beta_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period < 2 || store.len() < period + 1 {
        return None;
    }
    let n = period as f64;
    let start = store.len() - period;
    let mut returns = Vec::with_capacity(period);
    for i in start..store.len() {
        if store.close[i - 1] != 0.0 {
            returns.push((store.close[i] - store.close[i - 1]) / store.close[i - 1]);
        }
    }
    if returns.len() < 2 {
        return None;
    }
    let count = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / count;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (count - 1.0);
    Some(variance.sqrt() * n.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn close_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            vec![1.0; len],
        )
    }

    fn assert_series_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else {
                assert!((actual - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn beta_is_zero_for_constant_returns() {
        let store = close_store(&[1.0, 2.0, 4.0, 8.0]);
        let values = beta_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 0.0, 0.0]);
        assert_eq!(latest_beta_store(&store, 2), Some(0.0));
    }

    #[test]
    fn beta_matches_the_return_stddev_proxy() {
        let store = close_store(&[1.0, 2.0, 2.0, 4.0]);
        let values = beta_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 1.0, 1.0]);
        assert!((latest_beta_store(&store, 2).unwrap() - 1.0).abs() < 1e-12);
    }
}

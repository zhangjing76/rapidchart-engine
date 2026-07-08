use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Historical Volatility: Annualized standard deviation of log returns over period.
/// HV = stddev(ln(close[i]/close[i-1]), period) * sqrt(252)
/// (252 for daily bars; for other timeframes the annualization factor stays the same
/// as a convention, since users interpret it relative to their timeframe.)
pub fn historical_volatility_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("hv:close:{period}");
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
    // Compute log returns
    let mut log_returns = vec![f64::NAN; len];
    for i in 1..len {
        if store.close[i - 1] > 0.0 && store.close[i] > 0.0 {
            log_returns[i] = (store.close[i] / store.close[i - 1]).ln();
        }
    }
    // Rolling stddev of log returns
    let annualize = (252.0f64).sqrt();
    for i in period..len {
        let window = &log_returns[i + 1 - period..=i];
        let valid: Vec<f64> = window.iter().filter(|v| !v.is_nan()).copied().collect();
        if valid.len() < 2 {
            continue;
        }
        let n = valid.len() as f64;
        let mean = valid.iter().sum::<f64>() / n;
        let variance = valid.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        out[i] = variance.sqrt() * annualize * 100.0; // as percentage
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_historical_volatility_store(store: &CandleStore, period: usize) -> Option<f64> {
    historical_volatility_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
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
    fn historical_volatility_is_zero_for_flat_prices() {
        let store = close_store(&[1.0, 1.0, 1.0]);
        let values = historical_volatility_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 0.0]);
        assert_eq!(latest_historical_volatility_store(&store, 2), Some(0.0));
    }

    #[test]
    fn historical_volatility_is_zero_for_constant_log_returns() {
        let store = close_store(&[1.0, 2.0, 4.0]);
        let values = historical_volatility_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 0.0]);
        assert_eq!(latest_historical_volatility_store(&store, 2), Some(0.0));
    }
}

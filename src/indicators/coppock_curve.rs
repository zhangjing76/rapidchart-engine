use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Coppock Curve: WMA(10) of (ROC(14) + ROC(11))
/// A momentum indicator originally designed for monthly charts.
pub fn coppock_curve_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "coppock:close".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    // Need at least 14 bars for ROC(14)
    if len < 15 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // Compute ROC(14) + ROC(11) series
    let mut roc_sum = vec![f64::NAN; len];
    for i in 14..len {
        let roc14 = if store.close[i - 14] != 0.0 {
            ((store.close[i] - store.close[i - 14]) / store.close[i - 14]) * 100.0
        } else {
            f64::NAN
        };
        let roc11 = if i >= 11 && store.close[i - 11] != 0.0 {
            ((store.close[i] - store.close[i - 11]) / store.close[i - 11]) * 100.0
        } else {
            f64::NAN
        };
        if !roc14.is_nan() && !roc11.is_nan() {
            roc_sum[i] = roc14 + roc11;
        }
    }
    // WMA(10) of roc_sum
    let wma_period = 10;
    let weight_sum: f64 = (1..=wma_period).map(|w| w as f64).sum();
    for i in 0..len {
        if i + 1 < wma_period {
            continue;
        }
        let window = &roc_sum[i + 1 - wma_period..=i];
        if window.iter().any(|v| v.is_nan()) {
            continue;
        }
        let weighted: f64 = window
            .iter()
            .enumerate()
            .map(|(j, &v)| v * (j + 1) as f64)
            .sum();
        out[i] = weighted / weight_sum;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_coppock_curve_store(store: &CandleStore) -> Option<f64> {
    coppock_curve_store(store, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn coppock_curve_is_constant_for_geometric_growth() {
        let values: Vec<f64> = (0..24).map(|i| 2f64.powi(i)).collect();
        let store = close_store(&values);
        let output = coppock_curve_store(&store, &mut HashMap::new());

        let mut expected = vec![f64::NAN; 24];
        expected[23] = 1_843_000.0;
        assert_series_close(&output, &expected);
        assert_eq!(latest_coppock_curve_store(&store), Some(1_843_000.0));
    }

    #[test]
    fn coppock_curve_is_the_wma_of_the_two_long_rocs() {
        let values: Vec<f64> = (1..=24).map(|v| v as f64).collect();
        let store = close_store(&values);
        let output = coppock_curve_store(&store, &mut HashMap::new());

        let mut expected = vec![f64::NAN; 24];
        expected[23] = 373.73742923742924;
        assert_series_close(&output, &expected);
        assert!((latest_coppock_curve_store(&store).unwrap() - 373.73742923742924).abs() < 1e-12);
    }
}

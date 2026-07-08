use crate::NodeCache;
use crate::{CandleStore, Series};
use std::collections::HashMap;

/// Smoothed Moving Average (Wilder's smoothing): alpha = 1/period
fn smma(values: &[f64], period: usize) -> Series {
    let alpha = 1.0 / period as f64;
    let mut out = Vec::with_capacity(values.len());
    let mut current = None::<f64>;
    for &v in values {
        if v.is_nan() {
            out.push(f64::NAN);
            continue;
        }
        let next = match current {
            Some(prev) => alpha * v + (1.0 - alpha) * prev,
            None => v,
        };
        current = Some(next);
        out.push(next);
    }
    out
}

/// Gator Oscillator: derived from Alligator's jaw, teeth, lips.
/// Upper histogram = |jaw_smma - teeth_smma| (positive)
/// Lower histogram = -(|teeth_smma - lips_smma|) (negative)

pub fn gator_oscillator_store(
    store: &CandleStore,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let median: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(h, l)| (h + l) / 2.0)
        .collect();
    let jaw = smma(&median, 13);
    let teeth = smma(&median, 8);
    let lips = smma(&median, 5);
    let len = store.len();
    let mut upper = vec![f64::NAN; len];
    let mut lower = vec![f64::NAN; len];
    for i in 0..len {
        if !jaw[i].is_nan() && !teeth[i].is_nan() {
            upper[i] = (jaw[i] - teeth[i]).abs();
        }
        if !teeth[i].is_nan() && !lips[i].is_nan() {
            lower[i] = -(teeth[i] - lips[i]).abs();
        }
    }
    vec![
        crate::named_series("upper", upper),
        crate::named_series("lower", lower),
    ]
}

pub fn latest_gator_oscillator_store(store: &CandleStore) -> (Option<f64>, Option<f64>) {
    let outputs = gator_oscillator_store(store, &mut HashMap::new());
    let upper = outputs[0]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    let lower = outputs[1]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    (upper, lower)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[f64]) -> CandleStore {
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
    fn gator_oscillator_is_zero_for_constant_prices() {
        let store = ohlc_store(&[10.0; 6]);
        let values = gator_oscillator_store(&store, &mut HashMap::new());

        assert_series_close(&values[0].values, &[0.0; 6]);
        assert_series_close(&values[1].values, &[0.0; 6]);
        let latest = latest_gator_oscillator_store(&store);
        assert!((latest.0.unwrap() - 0.0).abs() < 1e-12);
        assert!((latest.1.unwrap() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn gator_oscillator_measures_spread_between_smoothed_lines() {
        let store = ohlc_store(&(1..=14).map(|v| v as f64).collect::<Vec<_>>());
        let values = gator_oscillator_store(&store, &mut HashMap::new());

        assert!((values[0].values[13] - 1.994578601098711).abs() < 1e-12);
        assert!((values[1].values[13] + 1.986221742802563).abs() < 1e-12);
        assert_eq!(
            latest_gator_oscillator_store(&store),
            (Some(1.994578601098711), Some(-1.986221742802563))
        );
    }
}

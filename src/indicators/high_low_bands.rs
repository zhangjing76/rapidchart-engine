use crate::NodeCache;
use crate::{CandleStore, Series};

/// Simple Moving Average over a slice of f64 values.
fn sma_series(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    let mut sum: f64 = values[..period].iter().sum();
    out[period - 1] = sum / period as f64;
    for i in period..values.len() {
        sum += values[i] - values[i - period];
        out[i] = sum / period as f64;
    }
    out
}

/// High Low Bands: SMA of highs (upper) of lows (lower), midpoint of upper+lower (middle).

pub fn high_low_bands_store(
    store: &CandleStore,
    period: usize,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let upper = sma_series(&store.high, period);
    let lower = sma_series(&store.low, period);
    let middle: Vec<f64> = upper
        .iter()
        .zip(lower.iter())
        .map(|(u, l)| {
            if u.is_nan() || l.is_nan() {
                f64::NAN
            } else {
                (u + l) / 2.0
            }
        })
        .collect();
    vec![
        crate::named_series("upper", upper),
        crate::named_series("middle", middle),
        crate::named_series("lower", lower),
    ]
}

pub fn latest_high_low_bands_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let start = store.len() - period;
    let upper: f64 = store.high[start..].iter().sum::<f64>() / period as f64;
    let lower: f64 = store.low[start..].iter().sum::<f64>() / period as f64;
    let middle = (upper + lower) / 2.0;
    (Some(upper), Some(middle), Some(lower))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
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
    fn high_low_bands_are_the_rolling_mean_of_highs_and_lows() {
        let store = ohlc_store(&[(4.0, 0.0, 2.0), (6.0, 2.0, 4.0), (8.0, 4.0, 6.0)]);
        let outputs = high_low_bands_store(&store, 2, &mut HashMap::new());

        assert_series_close(&outputs[0].values, &[f64::NAN, 5.0, 7.0]);
        assert_series_close(&outputs[1].values, &[f64::NAN, 3.0, 5.0]);
        assert_series_close(&outputs[2].values, &[f64::NAN, 1.0, 3.0]);
        assert_eq!(latest_high_low_bands_store(&store, 2), (Some(7.0), Some(5.0), Some(3.0)));
    }
}

use crate::indicators::sma::sma_close_store;
use crate::series::rc_into_owned;
use crate::CandleStore;
use crate::NodeCache;

/// Moving Average Cross: Two SMAs (fast and slow) with a difference histogram.
/// Outputs: fast, slow, histogram (fast - slow).

pub fn ma_cross_store(
    store: &CandleStore,
    fast_period: usize,
    slow_period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let fast = rc_into_owned(sma_close_store(store, fast_period, nodes));
    let slow = rc_into_owned(sma_close_store(store, slow_period, nodes));
    let histogram: Vec<f64> = fast
        .iter()
        .zip(slow.iter())
        .map(|(f, s)| {
            if f.is_nan() || s.is_nan() {
                f64::NAN
            } else {
                f - s
            }
        })
        .collect();
    vec![
        crate::named_series("fast", fast),
        crate::named_series("slow", slow),
        crate::named_series("histogram", histogram),
    ]
}

pub fn latest_ma_cross_store(
    store: &CandleStore,
    fast_period: usize,
    slow_period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let fast = crate::indicators::sma::latest_sma_store(store, fast_period);
    let slow = crate::indicators::sma::latest_sma_store(store, slow_period);
    let histogram = match (fast, slow) {
        (Some(f), Some(s)) => Some(f - s),
        _ => None,
    };
    (fast, slow, histogram)
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
    fn ma_cross_histogram_is_the_difference_of_the_two_smas() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0]);
        let outputs = ma_cross_store(&store, 2, 3, &mut HashMap::new());

        assert_series_close(&outputs[0].values, &[f64::NAN, 1.5, 2.5, 3.5]);
        assert_series_close(&outputs[1].values, &[f64::NAN, f64::NAN, 2.0, 3.0]);
        assert_series_close(&outputs[2].values, &[f64::NAN, f64::NAN, 0.5, 0.5]);
        let (fast, slow, histogram) = latest_ma_cross_store(&store, 2, 3);
        assert!((fast.unwrap() - 3.5).abs() < 1e-12);
        assert!((slow.unwrap() - 3.0).abs() < 1e-12);
        assert!((histogram.unwrap() - 0.5).abs() < 1e-12);
    }
}

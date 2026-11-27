use crate::indicators::kst::sma_from_series;
use crate::indicators::sma::sma_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::rc::Rc;

/// Window lengths of the two simple averages that make a triangular average of `period` bars:
/// ceil(period / 2), then the same again, plus one when `period` is even. Together they span
/// `period` bars (TA-Lib and most charting packages use this definition).
pub fn triangular_windows(period: usize) -> (usize, usize) {
    let first = period.div_ceil(2);
    let second = if period % 2 == 0 { first + 1 } else { first };
    (first, second)
}

/// Triangular moving average of `values` over `period` bars.
pub fn triangular_series(values: &[f64], period: usize) -> Series {
    let (first, second) = triangular_windows(period);
    sma_from_series(&sma_from_series(values, first), second)
}

/// Latest triangular moving average of `values`, from the last bars it depends on.
pub fn latest_triangular(values: &[f64], period: usize) -> Option<f64> {
    let (first, second) = triangular_windows(period);
    let needed = first + second - 1;
    if period == 0 || values.len() < needed {
        return None;
    }
    triangular_series(&values[values.len() - needed..], period)
        .last()
        .copied()
        .filter(|v| !v.is_nan())
}

pub fn trima_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("trima:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let (first, second) = triangular_windows(period);
    let values = sma_from_series(&sma_close_store(store, first, nodes), second);
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_trima_store(store: &CandleStore, period: usize) -> Option<f64> {
    latest_triangular(&store.close, period)
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("TRIMA", "TRIMA", "Moving Average", "overlay", 50)
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
    fn triangular_windows_match_the_usual_definition() {
        assert_eq!(triangular_windows(50), (25, 26));
        assert_eq!(triangular_windows(10), (5, 6));
        assert_eq!(triangular_windows(4), (2, 3));
        assert_eq!(triangular_windows(3), (2, 2));
        assert_eq!(triangular_windows(1), (1, 1));
    }

    #[test]
    fn trima_is_the_input_when_prices_are_constant() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0, 10.0]);
        let values = trima_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 10.0, 10.0, 10.0]);
        assert_eq!(latest_trima_store(&store, 3), Some(10.0));
    }

    #[test]
    fn trima_spans_period_bars() {
        // Period 3: SMA(2) of the close, then SMA(2) of that, so the first value is at bar 2.
        let store = close_store(&[10.0, 12.0, 14.0, 16.0, 18.0]);
        let values = trima_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 12.0, 14.0, 16.0]);
        assert_eq!(latest_trima_store(&store, 3), Some(16.0));
    }

    #[test]
    fn trima_uses_a_longer_second_average_for_an_even_period() {
        // Period 4: SMA(2), then SMA(3).
        let store = close_store(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let nan = f64::NAN;
        let values = trima_store(&store, 4, &mut HashMap::new());

        assert_series_close(&values, &[nan, nan, nan, 2.5, 3.5, 4.5]);
        assert_eq!(latest_trima_store(&store, 4), Some(4.5));
    }

    #[test]
    fn latest_trima_matches_the_full_calculation() {
        let closes: Vec<f64> = (0..60).map(|i| 50.0 + ((i * 7) % 11) as f64).collect();
        let store = close_store(&closes);
        for period in [1, 2, 3, 10, 11, 50] {
            let full = trima_store(&store, period, &mut HashMap::new());
            let latest = latest_trima_store(&store, period).unwrap();
            assert!((latest - full[59]).abs() < 1e-9, "period {period}");
        }
        assert_eq!(latest_trima_store(&close_store(&[1.0, 2.0]), 4), None);
    }
}

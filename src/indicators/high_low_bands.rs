use crate::indicators::trima::{latest_triangular, triangular_series};
use crate::CandleStore;
use crate::NodeCache;
/// High Low Bands: a triangular moving average of the close (middle) with bands a fixed
/// percentage above and below it. `shift_percent` is the band offset in percent, e.g. 5.0.
pub fn high_low_bands_store(
    store: &CandleStore,
    period: usize,
    shift_percent: f64,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let middle = triangular_series(&store.close, period);
    let shift = shift_percent / 100.0;
    let upper: Vec<f64> = middle.iter().map(|m| m * (1.0 + shift)).collect();
    let lower: Vec<f64> = middle.iter().map(|m| m * (1.0 - shift)).collect();
    vec![
        crate::named_series("upper", upper),
        crate::named_series("middle", middle),
        crate::named_series("lower", lower),
    ]
}

pub fn latest_high_low_bands_store(
    store: &CandleStore,
    period: usize,
    shift_percent: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_triangular(&store.close, period);
    let shift = shift_percent / 100.0;
    match middle {
        Some(m) => (Some(m * (1.0 + shift)), Some(m), Some(m * (1.0 - shift))),
        None => (None, None, None),
    }
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "HIGH_LOW_BANDS",
        name: "HIGH LOW BANDS",
        category: "Statistical",
        pane: "overlay",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "period",
                label: "Period",
                default: 10.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "multiplier",
                label: "Shift %",
                default: 5.0,
                min: 0.1,
                step: "0.1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "line", "overlay", "#f59e0b"),
            crate::descriptors::output_descriptor("middle", "line", "overlay", "#64748b"),
            crate::descriptors::output_descriptor("lower", "line", "overlay", "#f59e0b"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn close_store(closes: &[f64]) -> CandleStore {
        let len = closes.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            closes.to_vec(),
            closes.iter().map(|c| c + 1.0).collect(),
            closes.iter().map(|c| c - 1.0).collect(),
            closes.to_vec(),
            vec![1.0; len],
        )
    }

    fn assert_series_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan(), "expected NaN, got {actual}");
            } else {
                assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
            }
        }
    }

    #[test]
    fn high_low_bands_shift_a_triangular_average_of_the_close() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        // Period 4: SMA(2) of the close, then SMA(3) of that.
        let outputs = high_low_bands_store(&store, 4, 10.0, &mut HashMap::new());
        let nan = f64::NAN;

        assert_series_close(&outputs[1].values, &[nan, nan, nan, 2.5, 3.5, 4.5]);
        assert_series_close(&outputs[0].values, &[nan, nan, nan, 2.75, 3.85, 4.95]);
        assert_series_close(&outputs[2].values, &[nan, nan, nan, 2.25, 3.15, 4.05]);
    }

    #[test]
    fn high_low_bands_use_equal_windows_for_an_odd_period() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0]);
        let outputs = high_low_bands_store(&store, 3, 5.0, &mut HashMap::new());
        let nan = f64::NAN;
        assert_series_close(&outputs[1].values, &[nan, nan, 2.0, 3.0]);
    }

    #[test]
    fn latest_high_low_bands_match_the_full_calculation() {
        let closes: Vec<f64> = (0..40).map(|i| 50.0 + ((i * 7) % 11) as f64).collect();
        let store = close_store(&closes);
        for period in [1, 2, 3, 10, 11] {
            let outputs = high_low_bands_store(&store, period, 5.0, &mut HashMap::new());
            let (upper, middle, lower) = latest_high_low_bands_store(&store, period, 5.0);
            assert!((upper.unwrap() - outputs[0].values[39]).abs() < 1e-9);
            assert!((middle.unwrap() - outputs[1].values[39]).abs() < 1e-9);
            assert!((lower.unwrap() - outputs[2].values[39]).abs() < 1e-9);
        }
        assert_eq!(
            latest_high_low_bands_store(&close_store(&[1.0]), 10, 5.0),
            (None, None, None)
        );
    }
}

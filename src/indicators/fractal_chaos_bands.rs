use crate::nan_to_none;
use crate::CandleStore;
use crate::NodeCache;

/// Fractal Chaos Bands.
/// A fractal high occurs when a bar's high is the highest among
/// the 2 bars on each side (5-bar window). The upper band holds the most
/// recent fractal high. Lower band holds the most recent fractal low.

pub fn fractal_chaos_bands_store(
    store: &CandleStore,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let len = store.len();
    let mut upper = vec![f64::NAN; len];
    let mut lower = vec![f64::NAN; len];

    let mut last_fractal_high = f64::NAN;
    let mut last_fractal_low = f64::NAN;

    for i in 0..len {
        if i >= 4 {
            let mid = i - 2;
            let is_fractal_high = store.high[mid] > store.high[mid - 2]
                && store.high[mid] > store.high[mid - 1]
                && store.high[mid] > store.high[mid + 1]
                && store.high[mid] > store.high[mid + 2];
            if is_fractal_high {
                last_fractal_high = store.high[mid];
            }

            let is_fractal_low = store.low[mid] < store.low[mid - 2]
                && store.low[mid] < store.low[mid - 1]
                && store.low[mid] < store.low[mid + 1]
                && store.low[mid] < store.low[mid + 2];
            if is_fractal_low {
                last_fractal_low = store.low[mid];
            }
        }
        upper[i] = last_fractal_high;
        lower[i] = last_fractal_low;
    }

    vec![
        crate::named_series("upper", upper),
        crate::named_series("lower", lower),
    ]
}

pub fn latest_fractal_chaos_bands_store(store: &CandleStore) -> (Option<f64>, Option<f64>) {
    let len = store.len();
    if len < 5 {
        return (None, None);
    }

    let mut last_fractal_high = f64::NAN;
    let mut last_fractal_low = f64::NAN;

    for i in 4..len {
        let mid = i - 2;
        let is_fractal_high = store.high[mid] > store.high[mid - 2]
            && store.high[mid] > store.high[mid - 1]
            && store.high[mid] > store.high[mid + 1]
            && store.high[mid] > store.high[mid + 2];
        if is_fractal_high {
            last_fractal_high = store.high[mid];
        }
        let is_fractal_low = store.low[mid] < store.low[mid - 2]
            && store.low[mid] < store.low[mid - 1]
            && store.low[mid] < store.low[mid + 1]
            && store.low[mid] < store.low[mid + 2];
        if is_fractal_low {
            last_fractal_low = store.low[mid];
        }
    }

    (
        nan_to_none(last_fractal_high),
        nan_to_none(last_fractal_low),
    )
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "FRACTAL_CHAOS_BANDS",
        name: "FRACTAL CHAOS BANDS",
        category: "Trend Analysis",
        pane: "overlay",
        params: Vec::new(),
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "line", "overlay", "#9333ea"),
            crate::descriptors::output_descriptor("lower", "line", "overlay", "#9333ea"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ohlc_store(values: &[(f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, low)| *low).collect(),
            values.iter().map(|(high, _)| *high).collect(),
            values.iter().map(|(_, low)| *low).collect(),
            values.iter().map(|(_, low)| *low).collect(),
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
    fn fractal_chaos_bands_track_the_last_fractal_high() {
        let store = ohlc_store(&[(1.0, 1.0), (3.0, 1.0), (5.0, 1.0), (3.0, 1.0), (1.0, 1.0)]);
        let values = fractal_chaos_bands_store(&store, &mut std::collections::HashMap::new());

        assert_series_close(
            &values[0].values,
            &[f64::NAN, f64::NAN, f64::NAN, f64::NAN, 5.0],
        );
        assert_series_close(
            &values[1].values,
            &[f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN],
        );
        assert_eq!(latest_fractal_chaos_bands_store(&store), (Some(5.0), None));
    }
}

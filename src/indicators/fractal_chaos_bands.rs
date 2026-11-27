use crate::nan_to_none;
use crate::CandleStore;
use crate::NodeCache;

/// Whether bar `mid` is a fractal of `values`: more extreme than the two bars after it and than
/// the two nearest less-extreme bars before it. Earlier bars that tie bar `mid` are skipped, and
/// any more-extreme bar before those two rules it out. `sign` is 1.0 for highs and -1.0 for lows.
/// The fractal is confirmed at bar `mid + 2`, which must exist.
pub(crate) fn is_fractal(values: &[f64], mid: usize, sign: f64) -> bool {
    let centre = sign * values[mid];
    if !(sign * values[mid + 1] < centre && sign * values[mid + 2] < centre) {
        return false;
    }
    let mut less_extreme = 0;
    for value in values[..mid].iter().rev() {
        let value = sign * value;
        if value > centre {
            return false;
        }
        if value < centre {
            less_extreme += 1;
            if less_extreme == 2 {
                return true;
            }
        }
    }
    false
}

/// Fractal Chaos Bands. The upper band holds the most recent fractal high and the lower band the
/// most recent fractal low (see [`is_fractal`]), each from the bar that confirms it.
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
            if is_fractal(&store.high, mid, 1.0) {
                last_fractal_high = store.high[mid];
            }
            if is_fractal(&store.low, mid, -1.0) {
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
    let last_high = (2..len - 2)
        .rev()
        .find(|&mid| is_fractal(&store.high, mid, 1.0))
        .map(|mid| store.high[mid]);
    let last_low = (2..len - 2)
        .rev()
        .find(|&mid| is_fractal(&store.low, mid, -1.0))
        .map(|mid| store.low[mid]);
    (
        last_high.and_then(nan_to_none),
        last_low.and_then(nan_to_none),
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

    #[test]
    fn earlier_bars_that_tie_the_fractal_are_skipped() {
        let highs = [1.0, 2.0, 5.0, 5.0, 3.0, 1.0];
        assert!(is_fractal(&highs, 3, 1.0));
        // A tie after the centre bar rules it out.
        assert!(!is_fractal(&[1.0, 2.0, 5.0, 5.0, 1.0], 2, 1.0));
        // A higher bar before the two lower ones rules it out.
        assert!(!is_fractal(&[6.0, 2.0, 5.0, 5.0, 3.0, 1.0], 3, 1.0));
        // Lows use the mirror image.
        assert!(is_fractal(&[9.0, 8.0, 2.0, 2.0, 4.0, 6.0], 3, -1.0));
    }
}

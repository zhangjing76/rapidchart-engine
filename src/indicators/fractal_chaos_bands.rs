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

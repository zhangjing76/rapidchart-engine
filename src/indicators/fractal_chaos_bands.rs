use crate::nan_to_none;
use crate::NodeCache;
use crate::{Bar, CandleStore, IndicatorOutput};

/// Fractal Chaos Bands.
/// A fractal high occurs when a bar's high is the highest among
/// the 2 bars on each side (5-bar window). The upper band holds the most
/// recent fractal high. Lower band holds the most recent fractal low.
pub fn fractal_chaos_bands(
    bars: &[Bar],
    _nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let len = bars.len();
    let mut upper = vec![f64::NAN; len];
    let mut lower = vec![f64::NAN; len];

    let mut last_fractal_high = f64::NAN;
    let mut last_fractal_low = f64::NAN;

    // Need at least 5 bars to detect a fractal (index 2 is the middle)
    for i in 0..len {
        if i >= 4 {
            let mid = i - 2;
            // Check fractal high: bars[mid].high > all 4 neighbors
            let is_fractal_high = bars[mid].high > bars[mid - 2].high
                && bars[mid].high > bars[mid - 1].high
                && bars[mid].high > bars[mid + 1].high
                && bars[mid].high > bars[mid + 2].high;
            if is_fractal_high {
                last_fractal_high = bars[mid].high;
            }

            // Check fractal low: bars[mid].low < all 4 neighbors
            let is_fractal_low = bars[mid].low < bars[mid - 2].low
                && bars[mid].low < bars[mid - 1].low
                && bars[mid].low < bars[mid + 1].low
                && bars[mid].low < bars[mid + 2].low;
            if is_fractal_low {
                last_fractal_low = bars[mid].low;
            }
        }
        upper[i] = last_fractal_high;
        lower[i] = last_fractal_low;
    }

    vec![
        IndicatorOutput { name: "upper".to_string(), values: upper },
        IndicatorOutput { name: "lower".to_string(), values: lower },
    ]
}

pub fn fractal_chaos_bands_store(
    store: &CandleStore,
    _nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
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
        IndicatorOutput { name: "upper".to_string(), values: upper },
        IndicatorOutput { name: "lower".to_string(), values: lower },
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

    (nan_to_none(last_fractal_high), nan_to_none(last_fractal_low))
}

use crate::NodeCache;
use crate::{CandleStore, Series};
use std::rc::Rc;

pub fn stochastic_k_values(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    for index in period - 1..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        let highest = window.iter().copied().fold(f64::MIN, f64::max);
        let lowest = window.iter().copied().fold(f64::MAX, f64::min);
        let range = highest - lowest;
        let current = values[index];
        out[index] = if range == 0.0 {
            0.0
        } else {
            100.0 * (current - lowest) / range
        };
    }
    out
}
pub fn smooth_series(values: &[f64], smooth: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if smooth == 0 {
        return out;
    }
    for (index, out_val) in out.iter_mut().enumerate() {
        if index + 1 < smooth {
            continue;
        }
        let window = &values[index + 1 - smooth..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        *out_val = window.iter().sum::<f64>() / smooth as f64;
    }
    out
}
pub fn stochastic_k_store(store: &CandleStore, period: usize) -> Series {
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return out;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
        let window = index + 1 - period..=index;
        let highest_high = window
            .clone()
            .map(|i| store.high[i])
            .fold(f64::MIN, f64::max);
        let lowest_low = window.map(|i| store.low[i]).fold(f64::MAX, f64::min);
        let range = highest_high - lowest_low;
        *item = if range == 0.0 {
            0.0
        } else {
            100.0 * (store.close[index] - lowest_low) / range
        };
    }
    out
}
/// Raw (fast) %K at `index`: where the close sits in the high-low range of the last `period` bars.
fn fast_k_at(store: &CandleStore, period: usize, index: usize) -> f64 {
    let window = index + 1 - period..=index;
    let highest_high = window
        .clone()
        .map(|i| store.high[i])
        .fold(f64::MIN, f64::max);
    let lowest_low = window.map(|i| store.low[i]).fold(f64::MAX, f64::min);
    let range = highest_high - lowest_low;
    if range == 0.0 {
        0.0
    } else {
        100.0 * (store.close[index] - lowest_low) / range
    }
}

/// Full stochastic: %K is the `smooth`-bar average of the raw %K and %D is the `signal`-bar
/// average of %K. With the usual 14, 3, 3 this is the slow stochastic; `smooth` = 1 gives the
/// fast stochastic.
pub fn stochastic_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    signal: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let fast = stochastic_k_store(store, period);
    let k = smooth_series(&fast, smooth);
    let d = smooth_series(&k, signal);
    let outputs = vec![crate::named_series("k", k), crate::named_series("d", d)];
    nodes.insert(
        format!("stoch:hlc:{period}:{smooth}:{signal}"),
        Rc::clone(&outputs[0].values),
    );
    outputs
}

/// Latest %K and %D, from the last `period + smooth + signal - 2` bars.
pub fn latest_stochastic_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    signal: usize,
) -> (Option<f64>, Option<f64>) {
    let len = store.len();
    if period == 0 || smooth == 0 || signal == 0 || len < period + smooth - 1 {
        return (None, None);
    }
    let k_at = |index: usize| {
        (index + 1 - smooth..=index)
            .map(|i| fast_k_at(store, period, i))
            .sum::<f64>()
            / smooth as f64
    };
    let k = k_at(len - 1);
    if len < period + smooth + signal - 2 {
        return (Some(k), None);
    }
    let d = (len - signal..len).map(k_at).sum::<f64>() / signal as f64;
    (Some(k), Some(d))
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "STOCHASTIC",
        name: "STOCHASTIC",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "period",
                label: "Period",
                default: 14.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "smooth",
                label: "%K Smoothing",
                default: 3.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "signal",
                label: "%D Period",
                default: 3.0,
                min: 1.0,
                step: "1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("k", "line", "separate", "#2563eb"),
            crate::descriptors::output_descriptor("d", "line", "separate", "#dc2626"),
        ],
    }
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
    fn stochastic_k_and_d_are_one_hundred_for_close_at_the_high() {
        let store = ohlc_store(&[
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
        ]);
        let values = stochastic_store(&store, 3, 2, 2, &mut HashMap::new());
        let nan = f64::NAN;

        assert_series_close(&values[0].values, &[nan, nan, nan, 100.0, 100.0]);
        assert_series_close(&values[1].values, &[nan, nan, nan, nan, 100.0]);
    }

    #[test]
    fn stochastic_k_is_the_smoothed_fast_k_and_d_averages_k() {
        // Period 2: fast %K is 100 when the close is at the 2-bar high and 0 at the low.
        let store = ohlc_store(&[
            (10.0, 0.0, 5.0),
            (10.0, 0.0, 10.0),
            (10.0, 0.0, 0.0),
            (10.0, 0.0, 10.0),
            (10.0, 0.0, 10.0),
        ]);
        let nan = f64::NAN;
        let fast = stochastic_store(&store, 2, 1, 1, &mut HashMap::new());
        assert_series_close(&fast[0].values, &[nan, 100.0, 0.0, 100.0, 100.0]);

        let slow = stochastic_store(&store, 2, 2, 2, &mut HashMap::new());
        assert_series_close(&slow[0].values, &[nan, nan, 50.0, 50.0, 100.0]);
        assert_series_close(&slow[1].values, &[nan, nan, nan, 50.0, 75.0]);
    }

    #[test]
    fn latest_stochastic_matches_the_full_calculation() {
        let bars: Vec<(f64, f64, f64)> = (0..40)
            .map(|i| {
                let mid = 50.0 + ((i * 7) % 11) as f64;
                (mid + 2.0, mid - 2.0, mid + ((i % 3) as f64 - 1.0))
            })
            .collect();
        let store = ohlc_store(&bars);
        for (period, smooth, signal) in [(14, 3, 3), (5, 1, 3), (3, 2, 4), (1, 1, 1)] {
            let full = stochastic_store(&store, period, smooth, signal, &mut HashMap::new());
            let (k, d) = latest_stochastic_store(&store, period, smooth, signal);
            assert!((k.unwrap() - full[0].values[39]).abs() < 1e-9);
            assert!((d.unwrap() - full[1].values[39]).abs() < 1e-9);
        }
        let short = ohlc_store(&bars[..15]);
        assert_eq!(latest_stochastic_store(&short, 14, 3, 3), (None, None));
    }
}

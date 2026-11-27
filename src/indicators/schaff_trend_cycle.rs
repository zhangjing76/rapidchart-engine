use crate::indicators::ema::ema_sma_seeded_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Schaff Trend Cycle, following Doug Schaff's original code:
/// 1. MACD line = EMA(fast) - EMA(slow), both seeded with a simple average
/// 2. %K of the MACD line over `stoch_period` bars, smoothed by a factor of 0.5
/// 3. %K of that smoothed line over `stoch_period` bars, smoothed again by 0.5 = STC
///
/// Each %K needs a full window, and a flat window repeats the previous %K (0 before the first).
pub fn schaff_trend_cycle_store(
    store: &CandleStore,
    fast: usize,
    slow: usize,
    stoch_period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("stc_seeded:{}:{}:{}", fast, slow, stoch_period);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema_fast = ema_sma_seeded_close_store(store, fast, nodes);
    let ema_slow = ema_sma_seeded_close_store(store, slow, nodes);
    let macd_line: Series = ema_fast
        .iter()
        .zip(ema_slow.iter())
        .map(|(fast, slow)| fast - slow)
        .collect();
    let smooth1 = smooth_half(&stochastic_holding(&macd_line, stoch_period));
    let out = smooth_half(&stochastic_holding(&smooth1, stoch_period));
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

/// %K of `values` over full windows of `period`: 100 * (value - lowest) / (highest - lowest).
/// A flat window repeats the previous %K, which starts at 0.
fn stochastic_holding(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 {
        return out;
    }
    let mut previous = 0.0;
    for i in period.saturating_sub(1)..values.len() {
        let window = &values[i + 1 - period..=i];
        if window.iter().any(|v| v.is_nan()) {
            continue;
        }
        let highest = window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = window.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        if highest > lowest {
            previous = 100.0 * (values[i] - lowest) / (highest - lowest);
        }
        out[i] = previous;
    }
    out
}

/// Smooths by a factor of 0.5, starting from the first value.
fn smooth_half(values: &[f64]) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    let mut previous = None::<f64>;
    for (i, &value) in values.iter().enumerate() {
        if value.is_nan() {
            continue;
        }
        let next = previous.map_or(value, |p| p + 0.5 * (value - p));
        previous = Some(next);
        out[i] = next;
    }
    out
}
pub fn latest_schaff_trend_cycle_store(
    store: &CandleStore,
    fast: usize,
    slow: usize,
    stoch_period: usize,
) -> Option<f64> {
    schaff_trend_cycle_store(store, fast, slow, stoch_period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "SCHAFF_TREND_CYCLE",
        name: "SCHAFF TREND CYCLE",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "fast",
                label: "Fast",
                default: 23.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "slow",
                label: "Slow",
                default: 50.0,
                min: 2.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "stoch_period",
                label: "Cycle",
                default: 10.0,
                min: 1.0,
                step: "1",
            },
        ],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#2563eb",
        )],
    }
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
    fn schaff_trend_cycle_is_zero_for_constant_prices() {
        let store = close_store(&[10.0; 6]);
        let values = schaff_trend_cycle_store(&store, 2, 3, 2, &mut HashMap::new());

        // The MACD line starts at bar 2 and each two-bar %K adds a bar; a flat MACD keeps the
        // initial 0.
        assert_series_close(&values, &[f64::NAN, f64::NAN, f64::NAN, f64::NAN, 0.0, 0.0]);
        assert_eq!(latest_schaff_trend_cycle_store(&store, 2, 3, 2), Some(0.0));
    }

    #[test]
    fn a_flat_stochastic_window_repeats_the_previous_value() {
        let values = stochastic_holding(&[1.0, 3.0, 2.0, 2.0, 2.0], 2);
        assert_series_close(&values, &[f64::NAN, 100.0, 0.0, 0.0, 0.0]);
        let values = stochastic_holding(&[1.0, 3.0, 3.0], 2);
        assert_series_close(&values, &[f64::NAN, 100.0, 100.0]);
    }

    #[test]
    fn the_smoothing_halves_the_gap_each_bar() {
        let values = smooth_half(&[f64::NAN, 100.0, 0.0, 0.0]);
        assert_series_close(&values, &[f64::NAN, 100.0, 50.0, 25.0]);
    }
}

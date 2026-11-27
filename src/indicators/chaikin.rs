use crate::indicators::adl::adl_store;
use crate::indicators::ema::ema_series;
use crate::indicators::kst::sma_from_series;
use crate::IndicatorArena;
use crate::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn chaikin_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("chaikin:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let adl_values = adl_store(store, nodes);
    let fast = ema_series(&adl_values, params.fast);
    let slow = ema_series(&adl_values, params.slow);
    let values: Vec<_> = fast
        .iter()
        .zip(slow.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() => *fast - *slow,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
/// Chaikin Volatility: the percent change over `roc` bars of a `period`-bar simple average of the
/// high-low range. With the defaults (14, 2) the first value is at bar 15.
pub fn chaikin_volatility_store(
    store: &CandleStore,
    period: usize,
    roc: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let key = format!("cvol_sma:value:{period}:{roc}");
    let ma_key = format!("cvol_sma:ma:{period}");
    if let (Some(value), Some(ma)) = (nodes.get(&key), nodes.get(&ma_key)) {
        return vec![
            crate::named_series("value", Rc::clone(value)),
            crate::named_series("hl_ma", Rc::clone(ma)),
        ];
    }
    let ranges: Vec<f64> = (0..store.len())
        .map(|index| store.high[index] - store.low[index])
        .collect();
    let ma = sma_from_series(&ranges, period);
    let mut values = vec![f64::NAN; store.len()];
    for index in roc..store.len() {
        values[index] = range_change(ma[index], ma[index - roc]);
    }
    nodes.insert(key, Rc::new(values.clone()));
    nodes.insert(ma_key, Rc::new(ma.clone()));
    vec![
        crate::named_series("value", values),
        crate::named_series("hl_ma", ma),
    ]
}

/// Percent change from `previous` to `current`; no value when `previous` is missing or 0.
fn range_change(current: f64, previous: f64) -> f64 {
    if current.is_nan() || previous.is_nan() || previous == 0.0 {
        f64::NAN
    } else {
        100.0 * (current / previous - 1.0)
    }
}

/// Simple average of the high-low range over the `period` bars ending at `index`.
fn range_average(store: &CandleStore, period: usize, index: usize) -> f64 {
    (index + 1 - period..=index)
        .map(|i| store.high[i] - store.low[i])
        .sum::<f64>()
        / period as f64
}

/// Latest Chaikin Volatility and range average, from the last `period + roc` bars.
pub fn latest_chaikin_volatility_store(
    store: &CandleStore,
    period: usize,
    roc: usize,
) -> (Option<f64>, Option<f64>) {
    let len = store.len();
    if period == 0 || len < period {
        return (None, None);
    }
    let current = range_average(store, period, len - 1);
    if len < period + roc {
        return (None, Some(current));
    }
    let previous = range_average(store, period, len - 1 - roc);
    let value = Some(range_change(current, previous)).filter(|v| !v.is_nan());
    (value, Some(current))
}
pub fn latest_chaikin_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if store.len() == 0 {
        return (None, None, None, None);
    }
    // Compute latest ADL value
    let i = store.len() - 1;
    let range = store.high[i] - store.low[i];
    let mfm = if range == 0.0 {
        0.0
    } else {
        ((store.close[i] - store.low[i]) - (store.high[i] - store.close[i])) / range
    };
    let mfv = mfm * store.volume[i];
    let prev_adl = outputs
        .get("adl")
        .and_then(|s| s.get(store.len().wrapping_sub(2)).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(0.0);
    let adl = if store.len() == 1 {
        mfv
    } else {
        prev_adl + mfv
    };
    // EMA steps
    let alpha_fast = 2.0 / (params.fast as f64 + 1.0);
    let alpha_slow = 2.0 / (params.slow as f64 + 1.0);
    let prev_fast = outputs
        .get("fast_ema")
        .and_then(|s| s.get(store.len().wrapping_sub(2)).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(adl);
    let fast_ema = if store.len() == 1 {
        adl
    } else {
        alpha_fast * adl + (1.0 - alpha_fast) * prev_fast
    };
    let prev_slow = outputs
        .get("slow_ema")
        .and_then(|s| s.get(store.len().wrapping_sub(2)).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(adl);
    let slow_ema = if store.len() == 1 {
        adl
    } else {
        alpha_slow * adl + (1.0 - alpha_slow) * prev_slow
    };
    (
        Some(fast_ema - slow_ema),
        Some(adl),
        Some(fast_ema),
        Some(slow_ema),
    )
}

pub(crate) fn chaikin_volatility_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "CHAIKIN_VOLATILITY",
        name: "CHAIKIN VOLATILITY",
        category: "Volatility",
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
                label: "Rate of Change",
                default: 2.0,
                min: 1.0,
                step: "1",
            },
        ],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#2563eb",
        )],
    }
}

pub(crate) fn chaikin_oscillator_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "CHAIKIN_OSCILLATOR",
        name: "CHAIKIN OSCILLATOR",
        category: "Money Flow",
        pane: "separate",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "fast",
                label: "Fast",
                default: 3.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "slow",
                label: "Slow",
                default: 10.0,
                min: 2.0,
                step: "1",
            },
        ],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#9333ea",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::named_series;
    use crate::types::IndicatorArena;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(high, _, _, _)| *high).collect(),
            values.iter().map(|(_, low, _, _)| *low).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(_, _, _, volume)| *volume).collect(),
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
    fn chaikin_oscillator_is_zero_when_adl_is_constant() {
        let store = ohlcv_store(&[
            (2.0, 0.0, 1.0, 10.0),
            (2.0, 0.0, 1.0, 10.0),
            (2.0, 0.0, 1.0, 10.0),
            (2.0, 0.0, 1.0, 10.0),
        ]);
        let values = chaikin_oscillator_store(
            &store,
            MacdParams {
                fast: 2,
                slow: 3,
                signal: 1,
            },
            &mut HashMap::new(),
        );

        assert_series_close(&values, &[0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn chaikin_oscillator_is_the_gap_between_fast_and_slow_adl_emas() {
        let store = ohlcv_store(&[
            (2.0, 0.0, 1.5, 10.0),
            (3.0, 1.0, 2.5, 10.0),
            (4.0, 2.0, 3.5, 10.0),
        ]);
        let params = MacdParams {
            fast: 2,
            slow: 3,
            signal: 1,
        };
        let values = chaikin_oscillator_store(&store, params, &mut HashMap::new());

        assert_series_close(&values, &[0.0, 0.8333333333333321, 1.5277777777777786]);
        let arena = IndicatorArena::from_named_outputs(vec![
            named_series("adl", vec![5.0, 10.0, 15.0]),
            named_series("fast_ema", vec![5.0, 8.333333333333332, 12.777777777777779]),
            named_series("slow_ema", vec![5.0, 7.5, 11.25]),
        ]);
        assert_eq!(
            latest_chaikin_oscillator_store(&store, params, &arena),
            (
                Some(1.5277777777777786),
                Some(15.0),
                Some(12.777777777777779),
                Some(11.25)
            )
        );
    }

    #[test]
    fn chaikin_volatility_is_zero_for_constant_ranges() {
        let store = ohlcv_store(&[
            (3.0, 1.0, 2.0, 10.0),
            (4.0, 2.0, 3.0, 10.0),
            (5.0, 3.0, 4.0, 10.0),
            (6.0, 4.0, 5.0, 10.0),
        ]);
        let values = chaikin_volatility_store(&store, 2, 1, &mut HashMap::new());

        assert_series_close(&values[0].values, &[f64::NAN, f64::NAN, 0.0, 0.0]);
    }

    #[test]
    fn chaikin_volatility_is_the_change_in_the_range_average() {
        // Ranges 2, 4, 8, 16; their 2-bar averages are 3, 6 and 12 from bar 1.
        let store = ohlcv_store(&[
            (3.0, 1.0, 2.0, 10.0),
            (6.0, 2.0, 4.0, 10.0),
            (10.0, 2.0, 6.0, 10.0),
            (18.0, 2.0, 10.0, 10.0),
        ]);
        let nan = f64::NAN;
        let one_bar = chaikin_volatility_store(&store, 2, 1, &mut HashMap::new());
        assert_series_close(&one_bar[0].values, &[nan, nan, 100.0, 100.0]);
        let two_bars = chaikin_volatility_store(&store, 2, 2, &mut HashMap::new());
        assert_series_close(&two_bars[0].values, &[nan, nan, nan, 300.0]);
        assert_eq!(
            latest_chaikin_volatility_store(&store, 2, 2),
            (Some(300.0), Some(12.0))
        );
    }

    #[test]
    fn latest_chaikin_volatility_matches_the_full_calculation() {
        let bars: Vec<(f64, f64, f64, f64)> = (0..40)
            .map(|i| {
                let mid = 50.0 + ((i * 7) % 11) as f64;
                let half = 1.0 + (i % 4) as f64;
                (mid + half, mid - half, mid, 10.0)
            })
            .collect();
        let full = chaikin_volatility_store(&ohlcv_store(&bars), 14, 2, &mut HashMap::new());
        for len in 1..=bars.len() {
            let (value, _) = latest_chaikin_volatility_store(&ohlcv_store(&bars[..len]), 14, 2);
            let expect = Some(full[0].values[len - 1]).filter(|v| !v.is_nan());
            match (value, expect) {
                (Some(a), Some(b)) => assert!((a - b).abs() < 1e-9, "len {len}"),
                (a, b) => assert_eq!(a, b, "len {len}"),
            }
        }
    }
}

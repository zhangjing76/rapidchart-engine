use crate::indicators::ema::{ema_next, ema_sma_seeded, ema_sma_seeded_close_store};
use crate::rc_into_owned;
use crate::IndicatorArena;
use crate::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::collections::HashMap;
use std::rc::Rc;

type MacdResult = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

const MACD_SLOT: usize = 0;
const SIGNAL_SLOT: usize = 1;
const HISTOGRAM_SLOT: usize = 2;
const FAST_EMA_SLOT: usize = 3;
const SLOW_EMA_SLOT: usize = 4;
/// MACD from averages that start with a simple average (see `ema_sma_seeded`): the MACD line
/// starts once the slow average has a value and the signal line `signal` bars after that.
pub fn macd_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let key = |part: &str| {
        format!(
            "macd_sma:{part}:{}:{}:{}",
            params.fast, params.slow, params.signal
        )
    };
    let fast_ema = rc_into_owned(ema_sma_seeded_close_store(store, params.fast, nodes));
    let slow_ema = rc_into_owned(ema_sma_seeded_close_store(store, params.slow, nodes));
    if let (Some(macd), Some(signal), Some(histogram)) = (
        nodes.get(&key("value")),
        nodes.get(&key("signal")),
        nodes.get(&key("histogram")),
    ) {
        return vec![
            crate::named_series("macd", (**macd).clone()),
            crate::named_series("signal", (**signal).clone()),
            crate::named_series("histogram", (**histogram).clone()),
            crate::named_series("fast_ema", fast_ema),
            crate::named_series("slow_ema", slow_ema),
        ];
    }
    let macd: Series = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(fast, slow)| fast - slow)
        .collect();
    let signal = ema_sma_seeded(&macd, params.signal);
    let histogram: Series = macd
        .iter()
        .zip(signal.iter())
        .map(|(macd, signal)| macd - signal)
        .collect();
    nodes.insert(key("value"), Rc::new(macd.clone()));
    nodes.insert(key("signal"), Rc::new(signal.clone()));
    nodes.insert(key("histogram"), Rc::new(histogram.clone()));
    vec![
        crate::named_series("macd", macd),
        crate::named_series("signal", signal),
        crate::named_series("histogram", histogram),
        crate::named_series("fast_ema", fast_ema),
        crate::named_series("slow_ema", slow_ema),
    ]
}

/// Latest MACD. Steps on from the previous bar's averages when they all have values;
/// otherwise (while the averages are starting) calculates them from the closes.
pub fn latest_macd_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> MacdResult {
    let Some(last) = store.last_close() else {
        return (None, None, None, None, None);
    };
    let previous = store.len().checked_sub(2).and_then(|index| {
        Some((
            outputs.value_at_slot(FAST_EMA_SLOT, index)?,
            outputs.value_at_slot(SLOW_EMA_SLOT, index)?,
            outputs.value_at_slot(SIGNAL_SLOT, index)?,
        ))
    });
    if let Some((fast, slow, signal)) = previous {
        let fast = ema_next(last, fast, params.fast);
        let slow = ema_next(last, slow, params.slow);
        let macd = fast - slow;
        let signal = ema_next(macd, signal, params.signal);
        return (
            Some(macd),
            Some(signal),
            Some(macd - signal),
            Some(fast),
            Some(slow),
        );
    }
    let full = macd_store(store, params, &mut HashMap::new());
    let at = |slot: usize| full[slot].values.last().copied().filter(|v| !v.is_nan());
    (
        at(MACD_SLOT),
        at(SIGNAL_SLOT),
        at(HISTOGRAM_SLOT),
        at(FAST_EMA_SLOT),
        at(SLOW_EMA_SLOT),
    )
}
/// The MACD line as a percentage of the slow average (0 when the slow average is 0).
fn ppo_value(macd: f64, slow: f64) -> f64 {
    if slow == 0.0 {
        0.0
    } else {
        100.0 * macd / slow
    }
}

/// PPO: the MACD line as a percentage of the slow average, with a signal line that starts like
/// MACD's. The fast and slow averages are kept as outputs so live updates can step on from them.
pub fn ppo_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let key = |part: &str| {
        format!(
            "ppo_sma:{part}:{}:{}:{}",
            params.fast, params.slow, params.signal
        )
    };
    let macd = macd_store(store, params, nodes);
    let fast_ema = macd[FAST_EMA_SLOT].values.clone();
    let slow_ema = macd[SLOW_EMA_SLOT].values.clone();
    if let (Some(ppo), Some(signal), Some(histogram)) = (
        nodes.get(&key("value")),
        nodes.get(&key("signal")),
        nodes.get(&key("histogram")),
    ) {
        return vec![
            crate::named_series("ppo", Rc::clone(ppo)),
            crate::named_series("signal", Rc::clone(signal)),
            crate::named_series("histogram", Rc::clone(histogram)),
            crate::named_series("fast_ema", fast_ema),
            crate::named_series("slow_ema", slow_ema),
        ];
    }
    let ppo: Series = macd[MACD_SLOT]
        .values
        .iter()
        .zip(slow_ema.iter())
        .map(|(&macd, &slow)| {
            if macd.is_nan() || slow.is_nan() {
                f64::NAN
            } else {
                ppo_value(macd, slow)
            }
        })
        .collect();
    let signal = ema_sma_seeded(&ppo, params.signal);
    let histogram: Series = ppo.iter().zip(signal.iter()).map(|(p, s)| p - s).collect();
    nodes.insert(key("value"), Rc::new(ppo.clone()));
    nodes.insert(key("signal"), Rc::new(signal.clone()));
    nodes.insert(key("histogram"), Rc::new(histogram.clone()));
    vec![
        crate::named_series("ppo", ppo),
        crate::named_series("signal", signal),
        crate::named_series("histogram", histogram),
        crate::named_series("fast_ema", fast_ema),
        crate::named_series("slow_ema", slow_ema),
    ]
}

/// Latest PPO. Steps on from the previous bar's averages and signal when they all have values;
/// otherwise (while they are starting) calculates them from the closes.
pub fn latest_ppo_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> MacdResult {
    let Some(last) = store.last_close() else {
        return (None, None, None, None, None);
    };
    let previous = store.len().checked_sub(2).and_then(|index| {
        Some((
            outputs.value_at_slot(FAST_EMA_SLOT, index)?,
            outputs.value_at_slot(SLOW_EMA_SLOT, index)?,
            outputs.value_at_slot(SIGNAL_SLOT, index)?,
        ))
    });
    if let Some((fast, slow, signal)) = previous {
        let fast = ema_next(last, fast, params.fast);
        let slow = ema_next(last, slow, params.slow);
        let ppo = ppo_value(fast - slow, slow);
        let signal = ema_next(ppo, signal, params.signal);
        return (
            Some(ppo),
            Some(signal),
            Some(ppo - signal),
            Some(fast),
            Some(slow),
        );
    }
    let full = ppo_store(store, params, &mut HashMap::new());
    let at = |slot: usize| full[slot].values.last().copied().filter(|v| !v.is_nan());
    (
        at(MACD_SLOT),
        at(SIGNAL_SLOT),
        at(HISTOGRAM_SLOT),
        at(FAST_EMA_SLOT),
        at(SLOW_EMA_SLOT),
    )
}
pub(crate) fn macd_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "MACD",
        name: "MACD",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "fast",
                label: "Fast",
                default: 12.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "slow",
                label: "Slow",
                default: 26.0,
                min: 2.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "signal",
                label: "Signal",
                default: 9.0,
                min: 1.0,
                step: "1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("macd", "line", "separate", "#2563eb"),
            crate::descriptors::output_descriptor("signal", "line", "separate", "#dc2626"),
            crate::descriptors::output_descriptor("histogram", "histogram", "separate", "#86efac"),
        ],
    }
}

pub(crate) fn ppo_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "PPO",
        name: "PPO",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "fast",
                label: "Fast",
                default: 12.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "slow",
                label: "Slow",
                default: 26.0,
                min: 2.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "signal",
                label: "Signal",
                default: 9.0,
                min: 1.0,
                step: "1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("ppo", "line", "separate", "#2563eb"),
            crate::descriptors::output_descriptor("signal", "line", "separate", "#dc2626"),
            crate::descriptors::output_descriptor("histogram", "histogram", "separate", "#86efac"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::named_series;
    use crate::types::IndicatorArena;
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

    #[test]
    fn macd_starts_once_the_averages_have_values() {
        let closes: Vec<f64> = (0..12).map(|i| 10.0 + i as f64).collect();
        let store = close_store(&closes);
        let params = MacdParams {
            fast: 2,
            slow: 4,
            signal: 3,
        };
        let outputs = macd_store(&store, params, &mut HashMap::new());
        let first = |slot: usize| outputs[slot].values.iter().position(|v| !v.is_nan());

        // The slow average starts at bar 3, so the MACD line does too; the signal 2 bars later.
        assert_eq!(first(MACD_SLOT), Some(3));
        assert_eq!(first(SIGNAL_SLOT), Some(5));
        assert_eq!(first(2), Some(5));
        // At bar 3: fast EMA(2) seeded at bar 1 with 10.5, then 11.5, 12.5; slow is (10..13)/4.
        assert!((outputs[MACD_SLOT].values[3] - (12.5 - 11.5)).abs() < 1e-12);
    }

    #[test]
    fn latest_macd_matches_the_full_calculation() {
        let closes: Vec<f64> = (0..60).map(|i| 50.0 + ((i * 7) % 11) as f64).collect();
        let params = MacdParams {
            fast: 12,
            slow: 26,
            signal: 9,
        };
        let full = macd_store(&close_store(&closes), params, &mut HashMap::new());
        for len in 1..=closes.len() {
            let store = close_store(&closes[..len]);
            let previous = IndicatorArena::from_named_outputs(
                full.iter()
                    .map(|o| named_series(&o.name, o.values[..len - 1].to_vec()))
                    .collect(),
            );
            let latest = latest_macd_store(&store, params, &previous);
            let expect = |slot: usize| Some(full[slot].values[len - 1]).filter(|v| !v.is_nan());
            let close = |a: Option<f64>, b: Option<f64>| match (a, b) {
                (Some(a), Some(b)) => (a - b).abs() < 1e-9,
                (a, b) => a == b,
            };
            assert!(close(latest.0, expect(MACD_SLOT)), "macd at {len}");
            assert!(close(latest.1, expect(SIGNAL_SLOT)), "signal at {len}");
            assert!(close(latest.2, expect(2)), "histogram at {len}");
        }
    }

    #[test]
    fn ppo_is_the_macd_line_as_a_percentage_of_the_slow_average() {
        let closes: Vec<f64> = (0..40).map(|i| 50.0 + ((i * 7) % 11) as f64).collect();
        let store = close_store(&closes);
        let params = MacdParams {
            fast: 3,
            slow: 6,
            signal: 4,
        };
        let macd = macd_store(&store, params, &mut HashMap::new());
        let ppo = ppo_store(&store, params, &mut HashMap::new());
        let first = |values: &[f64]| values.iter().position(|v| !v.is_nan());

        assert_eq!(first(&ppo[MACD_SLOT].values), Some(5));
        assert_eq!(first(&ppo[SIGNAL_SLOT].values), Some(8));
        for i in 5..40 {
            let expected = 100.0 * macd[MACD_SLOT].values[i] / macd[SLOW_EMA_SLOT].values[i];
            assert!((ppo[MACD_SLOT].values[i] - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn latest_ppo_matches_the_full_calculation() {
        let closes: Vec<f64> = (0..60).map(|i| 50.0 + ((i * 7) % 11) as f64).collect();
        let params = MacdParams {
            fast: 12,
            slow: 26,
            signal: 9,
        };
        let full = ppo_store(&close_store(&closes), params, &mut HashMap::new());
        for len in 1..=closes.len() {
            let store = close_store(&closes[..len]);
            let previous = IndicatorArena::from_named_outputs(
                full.iter()
                    .map(|o| named_series(&o.name, o.values[..len - 1].to_vec()))
                    .collect(),
            );
            let latest = latest_ppo_store(&store, params, &previous);
            let expect = |slot: usize| Some(full[slot].values[len - 1]).filter(|v| !v.is_nan());
            let close = |a: Option<f64>, b: Option<f64>| match (a, b) {
                (Some(a), Some(b)) => (a - b).abs() < 1e-9,
                (a, b) => a == b,
            };
            assert!(close(latest.0, expect(MACD_SLOT)), "ppo at {len}");
            assert!(close(latest.1, expect(SIGNAL_SLOT)), "signal at {len}");
            assert!(
                close(latest.2, expect(HISTOGRAM_SLOT)),
                "histogram at {len}"
            );
        }
    }
}

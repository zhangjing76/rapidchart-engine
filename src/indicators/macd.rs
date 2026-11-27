use crate::indicators::ema::{
    ema_close_store, ema_next, ema_series, ema_sma_seeded, ema_sma_seeded_close_store,
};
use crate::rc_into_owned;
use crate::IndicatorArena;
use crate::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, Series};
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
const FAST_EMA_SLOT: usize = 3;
const SLOW_EMA_SLOT: usize = 4;

pub fn macd_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let macd_key = format!(
        "macd:value:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let signal_key = format!(
        "macd:signal:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let histogram_key = format!(
        "macd:histogram:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let fast_key = format!("ema:close:{}", params.fast);
    let slow_key = format!("ema:close:{}", params.slow);
    if let (Some(macd), Some(signal), Some(histogram), Some(fast_ema), Some(slow_ema)) = (
        nodes.get(&macd_key),
        nodes.get(&signal_key),
        nodes.get(&histogram_key),
        nodes.get(&fast_key),
        nodes.get(&slow_key),
    ) {
        return vec![
            crate::named_series("macd", (**macd).clone()),
            crate::named_series("signal", (**signal).clone()),
            crate::named_series("histogram", (**histogram).clone()),
            crate::named_series("fast_ema", (**fast_ema).clone()),
            crate::named_series("slow_ema", (**slow_ema).clone()),
        ];
    }
    let fast_ema = rc_into_owned(ema_close_store(store, params.fast, nodes));
    let slow_ema = rc_into_owned(ema_close_store(store, params.slow, nodes));
    let macd: Series = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() => *fast - *slow,
            _ => f64::NAN,
        })
        .collect();
    let signal = ema_series(&macd, params.signal);
    let histogram: Series = macd
        .iter()
        .zip(signal.iter())
        .map(|(macd, signal)| match (macd, signal) {
            (macd, signal) if !macd.is_nan() && !signal.is_nan() => *macd - *signal,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(macd_key, Rc::new(macd.clone()));
    nodes.insert(signal_key, Rc::new(signal.clone()));
    nodes.insert(histogram_key, Rc::new(histogram.clone()));
    vec![
        crate::named_series("macd", macd),
        crate::named_series("signal", signal),
        crate::named_series("histogram", histogram),
        crate::named_series("fast_ema", fast_ema),
        crate::named_series("slow_ema", slow_ema),
    ]
}
pub fn latest_macd_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> MacdResult {
    let last = match store.last_close() {
        Some(last) => last,
        None => return (None, None, None, None, None),
    };
    if store.len() == 1 {
        return (Some(0.0), Some(0.0), Some(0.0), Some(last), Some(last));
    }
    let previous_index = store.len() - 2;
    let previous_close = store.close[previous_index];
    let previous_fast = outputs
        .value_at_slot(FAST_EMA_SLOT, previous_index)
        .unwrap_or(previous_close);
    let previous_slow = outputs
        .value_at_slot(SLOW_EMA_SLOT, previous_index)
        .unwrap_or(previous_close);
    let fast = ema_next(last, previous_fast, params.fast);
    let slow = ema_next(last, previous_slow, params.slow);
    let macd_line = fast - slow;
    let previous_signal = outputs
        .value_at_slot(SIGNAL_SLOT, previous_index)
        .unwrap_or(0.0);
    let signal = ema_next(macd_line, previous_signal, params.signal);
    let histogram = macd_line - signal;
    (
        Some(macd_line),
        Some(signal),
        Some(histogram),
        Some(fast),
        Some(slow),
    )
}
/// MACD from averages that start with a simple average (see `ema_sma_seeded`): the MACD line
/// starts once the slow average has a value and the signal line `signal` bars after that.
pub fn macd_seeded_store(
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

/// Latest seeded MACD. Steps on from the previous bar's averages when they all have values;
/// otherwise (while the averages are starting) calculates them from the closes.
pub fn latest_macd_seeded_store(
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
    let full = macd_seeded_store(store, params, &mut std::collections::HashMap::new());
    let at = |slot: usize| full[slot].values.last().copied().filter(|v| !v.is_nan());
    (
        at(MACD_SLOT),
        at(SIGNAL_SLOT),
        at(2),
        at(FAST_EMA_SLOT),
        at(SLOW_EMA_SLOT),
    )
}
pub fn ppo_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let ppo_key = format!(
        "ppo:value:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let signal_key = format!(
        "ppo:signal:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let histogram_key = format!(
        "ppo:histogram:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    if let (Some(ppo), Some(signal), Some(histogram)) = (
        nodes.get(&ppo_key),
        nodes.get(&signal_key),
        nodes.get(&histogram_key),
    ) {
        return vec![
            crate::named_series("ppo", Rc::clone(ppo)),
            crate::named_series("signal", Rc::clone(signal)),
            crate::named_series("histogram", Rc::clone(histogram)),
        ];
    }
    let macd_outputs = macd_store(store, params, nodes);
    let macd_line = macd_outputs[MACD_SLOT].values.clone();
    let slow_ema = macd_outputs[SLOW_EMA_SLOT].values.clone();
    let ppo: Series = macd_line
        .iter()
        .zip(slow_ema.iter())
        .map(|(macd, slow)| match (macd, slow) {
            (macd, slow) if !macd.is_nan() && !slow.is_nan() && *slow != 0.0 => {
                100.0 * *macd / *slow
            }
            (a, b) if !a.is_nan() && !b.is_nan() => 0.0,
            _ => f64::NAN,
        })
        .collect();
    let signal = ema_series(&ppo, params.signal);
    let histogram: Series = ppo
        .iter()
        .zip(signal.iter())
        .map(|(ppo, signal)| match (ppo, signal) {
            (ppo, signal) if !ppo.is_nan() && !signal.is_nan() => *ppo - *signal,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(ppo_key, Rc::new(ppo.clone()));
    nodes.insert(signal_key, Rc::new(signal.clone()));
    nodes.insert(histogram_key, Rc::new(histogram.clone()));
    vec![
        crate::named_series("ppo", ppo),
        crate::named_series("signal", signal),
        crate::named_series("histogram", histogram),
    ]
}
pub fn latest_ppo_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let (macd_line, _, _, _, slow_ema) = latest_macd_store(store, params, outputs);
    let ppo = match (macd_line, slow_ema) {
        (Some(macd_line), Some(slow_ema)) if slow_ema != 0.0 => Some(100.0 * macd_line / slow_ema),
        (Some(_), Some(_)) => Some(0.0),
        _ => None,
    };
    let previous_signal = store
        .len()
        .checked_sub(2)
        .and_then(|index| outputs.value_at_slot(SIGNAL_SLOT, index))
        .unwrap_or(ppo.unwrap_or(0.0));
    let signal = ppo.map(|ppo| ema_next(ppo, previous_signal, params.signal));
    let histogram = ppo.zip(signal).map(|(ppo, signal)| ppo - signal);
    (ppo, signal, histogram)
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
    fn macd_is_zero_for_constant_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let params = MacdParams {
            fast: 3,
            slow: 5,
            signal: 2,
        };
        let outputs = macd_store(&store, params, &mut HashMap::new());
        assert_eq!(&*outputs[0].values, &[0.0, 0.0, 0.0, 0.0]);
        assert_eq!(&*outputs[1].values, &[0.0, 0.0, 0.0, 0.0]);
        assert_eq!(&*outputs[2].values, &[0.0, 0.0, 0.0, 0.0]);

        let arena = IndicatorArena::from_named_outputs(vec![
            named_series("macd", vec![0.0, 0.0, 0.0, 0.0]),
            named_series("signal", vec![0.0, 0.0, 0.0, 0.0]),
            named_series("histogram", vec![0.0, 0.0, 0.0, 0.0]),
            named_series("fast_ema", vec![10.0, 10.0, 10.0, 10.0]),
            named_series("slow_ema", vec![10.0, 10.0, 10.0, 10.0]),
        ]);
        let latest = latest_macd_store(&store, params, &arena);
        assert_eq!(latest.0, Some(0.0));
        assert_eq!(latest.1, Some(0.0));
        assert_eq!(latest.2, Some(0.0));
    }

    #[test]
    fn macd_matches_manual_ema_differences_on_rising_prices() {
        let store = close_store(&[10.0, 12.0, 14.0, 16.0]);
        let params = MacdParams {
            fast: 3,
            slow: 5,
            signal: 2,
        };
        let outputs = macd_store(&store, params, &mut HashMap::new());

        assert_eq!(
            &*outputs[0].values,
            &[
                0.0,
                0.33333333333333215,
                0.7222222222222214,
                1.064814814814813
            ]
        );
        assert_eq!(
            &*outputs[1].values,
            &[
                0.0,
                0.22222222222222143,
                0.5555555555555548,
                0.8950617283950604
            ]
        );
        assert_eq!(
            &*outputs[2].values,
            &[
                0.0,
                0.11111111111111072,
                0.16666666666666663,
                0.16975308641975273
            ]
        );

        let arena = IndicatorArena::from_named_outputs(outputs);
        let latest = latest_macd_store(&store, params, &arena);
        assert_eq!(latest.0, Some(1.064814814814813));
        assert_eq!(latest.1, Some(0.8950617283950604));
        assert_eq!(latest.2, Some(0.16975308641975273));
        assert_eq!(latest.3, Some(14.25));
        assert_eq!(latest.4, Some(13.185185185185187));
    }

    #[test]
    fn seeded_macd_starts_once_the_averages_have_values() {
        let closes: Vec<f64> = (0..12).map(|i| 10.0 + i as f64).collect();
        let store = close_store(&closes);
        let params = MacdParams {
            fast: 2,
            slow: 4,
            signal: 3,
        };
        let outputs = macd_seeded_store(&store, params, &mut HashMap::new());
        let first = |slot: usize| outputs[slot].values.iter().position(|v| !v.is_nan());

        // The slow average starts at bar 3, so the MACD line does too; the signal 2 bars later.
        assert_eq!(first(MACD_SLOT), Some(3));
        assert_eq!(first(SIGNAL_SLOT), Some(5));
        assert_eq!(first(2), Some(5));
        // At bar 3: fast EMA(2) seeded at bar 1 with 10.5, then 11.5, 12.5; slow is (10..13)/4.
        assert!((outputs[MACD_SLOT].values[3] - (12.5 - 11.5)).abs() < 1e-12);
    }

    #[test]
    fn latest_seeded_macd_matches_the_full_calculation() {
        let closes: Vec<f64> = (0..60).map(|i| 50.0 + ((i * 7) % 11) as f64).collect();
        let params = MacdParams {
            fast: 12,
            slow: 26,
            signal: 9,
        };
        let full = macd_seeded_store(&close_store(&closes), params, &mut HashMap::new());
        for len in 1..=closes.len() {
            let store = close_store(&closes[..len]);
            let previous = IndicatorArena::from_named_outputs(
                full.iter()
                    .map(|o| named_series(&o.name, o.values[..len - 1].to_vec()))
                    .collect(),
            );
            let latest = latest_macd_seeded_store(&store, params, &previous);
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
}

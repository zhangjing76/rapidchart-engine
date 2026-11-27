use crate::indicators::ema::ema_sma_seeded;
use crate::types::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, NamedSeries, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Klinger Volume Oscillator:
/// signed volume = volume, negative when the typical price (H+L+C)/3 fell from the previous bar
/// KVO = EMA(fast) - EMA(slow) of the signed volume, 34 and 55 by default
/// signal = EMA(signal) of KVO, 13 by default
/// The EMAs start with a simple average. Bars with no volume are skipped.
pub(crate) const KLINGER_DEFAULTS: MacdParams = MacdParams {
    fast: 34,
    slow: 55,
    signal: 13,
};

#[cfg(test)]
pub fn klinger_volume_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    klinger_kvo_store(store, KLINGER_DEFAULTS.fast, KLINGER_DEFAULTS.slow, nodes)
}

fn klinger_kvo_store(
    store: &CandleStore,
    fast: usize,
    slow: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = if fast == KLINGER_DEFAULTS.fast && slow == KLINGER_DEFAULTS.slow {
        "klinger_sv:hlcv".to_string()
    } else {
        format!("klinger_sv:hlcv:{fast}:{slow}")
    };
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(klinger_series(store, fast, slow));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

/// Volume with the sign of the typical price's move from the previous bar. No value on the
/// first bar or where the volume is zero.
fn signed_volume(store: &CandleStore) -> Series {
    let typical = |i: usize| (store.high[i] + store.low[i] + store.close[i]) / 3.0;
    let mut out = vec![f64::NAN; store.len()];
    for (i, item) in out.iter_mut().enumerate().skip(1) {
        let volume = store.volume[i];
        if volume == 0.0 {
            continue;
        }
        *item = if typical(i) < typical(i - 1) {
            -volume
        } else {
            volume
        };
    }
    out
}

fn klinger_series(store: &CandleStore, fast: usize, slow: usize) -> Series {
    let volume = signed_volume(store);
    let ema_fast = ema_sma_seeded(&volume, fast);
    let ema_slow = ema_sma_seeded(&volume, slow);
    ema_fast
        .iter()
        .zip(ema_slow.iter())
        .map(|(f, s)| f - s)
        .collect()
}

/// KVO with its signal line.
pub fn klinger_outputs(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<NamedSeries> {
    let kvo = klinger_kvo_store(store, params.fast, params.slow, nodes);
    let signal = ema_sma_seeded(&kvo, params.signal);
    vec![
        crate::named_series("value", kvo),
        crate::named_series("signal", signal),
    ]
}
pub fn latest_klinger_outputs(
    store: &CandleStore,
    params: MacdParams,
) -> (Option<f64>, Option<f64>) {
    let outputs = klinger_outputs(store, params, &mut HashMap::new());
    let last = |index: usize| {
        outputs[index]
            .values
            .last()
            .copied()
            .and_then(|v| if v.is_nan() { None } else { Some(v) })
    };
    (last(0), last(1))
}

#[cfg(test)]
pub fn latest_klinger_volume_store(store: &CandleStore) -> Option<f64> {
    klinger_volume_store(store, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "KLINGER_VOLUME",
        name: "KLINGER VOLUME OSCILLATOR",
        category: "Money Flow",
        pane: "separate",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "fast",
                label: "Short Cycle",
                default: 34.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "slow",
                label: "Long Cycle",
                default: 55.0,
                min: 2.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "signal",
                label: "Signal",
                default: 13.0,
                min: 1.0,
                step: "1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("value", "line", "separate", "#2563eb"),
            crate::descriptors::output_descriptor("signal", "line", "separate", "#dc2626"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn signed_volume_follows_the_typical_price() {
        let store = ohlcv_store(&[
            (10.0, 8.0, 9.0, 5.0),
            (11.0, 9.0, 10.0, 6.0), // up
            (10.0, 8.0, 9.0, 7.0),  // down
            (10.0, 8.0, 9.0, 8.0),  // unchanged counts as up
            (12.0, 9.0, 10.0, 0.0), // no volume: skipped
        ]);
        let values = signed_volume(&store);
        assert!(values[0].is_nan() && values[4].is_nan());
        assert_eq!(&values[1..4], &[6.0, -7.0, 8.0]);
    }

    #[test]
    fn klinger_is_the_gap_between_the_signed_volume_averages() {
        // Up, up, down, up, up with volume 1 each, from bar 1.
        let store = ohlcv_store(&[
            (10.0, 10.0, 10.0, 1.0),
            (11.0, 11.0, 11.0, 1.0),
            (12.0, 12.0, 12.0, 1.0),
            (11.0, 11.0, 11.0, 1.0),
            (12.0, 12.0, 12.0, 1.0),
            (13.0, 13.0, 13.0, 1.0),
        ]);
        let params = MacdParams {
            fast: 2,
            slow: 3,
            signal: 2,
        };
        let outputs = klinger_outputs(&store, params, &mut HashMap::new());
        let nan = f64::NAN;
        // Signed volume: -, 1, 1, -1, 1, 1. EMA(2) from bar 2: 1, -1/3, 5/9, 23/27.
        // EMA(3) from bar 3: 1/3, 2/3, 5/6.
        let kvo = [
            nan,
            nan,
            nan,
            -1.0 / 3.0 - 1.0 / 3.0,
            5.0 / 9.0 - 2.0 / 3.0,
            23.0 / 27.0 - 5.0 / 6.0,
        ];
        for (got, want) in outputs[0].values.iter().zip(kvo) {
            assert!((want.is_nan() && got.is_nan()) || (got - want).abs() < 1e-12);
        }
        assert!(outputs[1].values[..4].iter().all(|v| v.is_nan()));
        let signal_first = (kvo[3] + kvo[4]) / 2.0;
        assert!((outputs[1].values[4] - signal_first).abs() < 1e-12);
    }

    #[test]
    fn klinger_is_zero_for_constant_prices_and_volume() {
        let store = ohlcv_store(&[(10.0, 10.0, 10.0, 5.0); 70]);
        let values = klinger_volume_store(&store, &mut HashMap::new());
        // The slow average starts after 55 signed volumes, from bar 1.
        assert!(values[..55].iter().all(|v| v.is_nan()));
        assert!(values[55..].iter().all(|&v| v == 0.0));
        assert_eq!(latest_klinger_volume_store(&store), Some(0.0));
    }
}

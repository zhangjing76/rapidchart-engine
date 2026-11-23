use crate::indicators::ema::ema_series;
use crate::types::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, NamedSeries, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Klinger Volume Oscillator:
/// Volume Force (VF) = volume * |2*(dm/cm) - 1| * trend * 100
/// where trend = +1 if (H+L+C) > prev(H+L+C), else -1
/// dm = high - low, cm = cumulative dm in same trend direction
/// KVO = EMA(fast, VF) - EMA(slow, VF), 34 and 55 by default; signal = EMA(signal, KVO), 13 by default.
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
        "klinger:hlcv".to_string()
    } else {
        format!("klinger:hlcv:{fast}:{slow}")
    };
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(klinger_series(store, fast, slow));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

fn klinger_series(store: &CandleStore, fast: usize, slow: usize) -> Series {
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 {
        return out;
    }
    let mut vf = vec![0.0f64; len];
    let mut cm = 0.0f64;
    let mut prev_trend = 1i8;
    for i in 1..len {
        let hlc = store.high[i] + store.low[i] + store.close[i];
        let prev_hlc = store.high[i - 1] + store.low[i - 1] + store.close[i - 1];
        let trend: i8 = if hlc > prev_hlc { 1 } else { -1 };
        let dm = store.high[i] - store.low[i];
        if trend == prev_trend {
            cm += dm;
        } else {
            cm = dm;
        }
        let ratio = if cm.abs() > 1e-10 {
            (2.0 * dm / cm) - 1.0
        } else {
            0.0
        };
        vf[i] = store.volume[i] * ratio.abs() * trend as f64 * 100.0;
        prev_trend = trend;
    }
    let ema_fast = ema_series(&vf, fast);
    let ema_slow = ema_series(&vf, slow);
    for i in 0..len {
        if !ema_fast[i].is_nan() && !ema_slow[i].is_nan() {
            out[i] = ema_fast[i] - ema_slow[i];
        }
    }
    out
}

/// KVO with its signal line.
pub fn klinger_outputs(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<NamedSeries> {
    let kvo = klinger_kvo_store(store, params.fast, params.slow, nodes);
    let signal = ema_series(&kvo, params.signal);
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
    fn klinger_volume_is_zero_for_constant_prices_and_volume() {
        let store = ohlcv_store(&[
            (10.0, 10.0, 10.0, 5.0),
            (10.0, 10.0, 10.0, 5.0),
            (10.0, 10.0, 10.0, 5.0),
        ]);
        let values = klinger_volume_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[0.0, 0.0, 0.0]);
        assert_eq!(latest_klinger_volume_store(&store), Some(0.0));
    }

    #[test]
    fn klinger_volume_tracks_the_gap_between_fast_and_slow_vf_emas() {
        let input: Vec<_> = (0..60)
            .map(|i| {
                let base = 10.0 + i as f64;
                (base + 1.0, base, base + 0.5, 1.0)
            })
            .collect();
        let store = ohlcv_store(&input);
        let values = klinger_volume_store(&store, &mut HashMap::new());

        assert!((values[59] - 9.580894095402968).abs() < 1e-12);
        assert_eq!(latest_klinger_volume_store(&store), Some(9.580894095402968));
    }
}

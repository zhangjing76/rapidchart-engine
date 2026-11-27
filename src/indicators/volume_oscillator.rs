use crate::indicators::ema::ema_sma_seeded;
use crate::types::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Volume Oscillator in points: fast EMA - slow EMA of the volume. The EMAs start with a simple
/// average, so the first value is at bar slow - 1.
pub fn volume_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("vol_osc_pts:volume:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let fast_ema = ema_sma_seeded(&store.volume, params.fast);
    let slow_ema = ema_sma_seeded(&store.volume, params.slow);
    let out: Vec<f64> = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(f, s)| f - s)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_volume_oscillator_store(store: &CandleStore, params: MacdParams) -> Option<f64> {
    volume_oscillator_store(store, params, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "VOLUME_OSCILLATOR",
        name: "VOLUME OSCILLATOR",
        category: "Volume",
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

    fn ohlcv_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
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
    fn volume_oscillator_is_zero_for_flat_volume() {
        let store = ohlcv_store(&[5.0, 5.0, 5.0]);
        let params = MacdParams {
            fast: 2,
            slow: 3,
            signal: 1,
        };
        let values = volume_oscillator_store(&store, params, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 0.0]);
        assert_eq!(latest_volume_oscillator_store(&store, params), Some(0.0));
    }

    #[test]
    fn volume_oscillator_tracks_fast_minus_slow_volume_ema() {
        let store = ohlcv_store(&[10.0, 20.0, 30.0]);
        let params = MacdParams {
            fast: 2,
            slow: 3,
            signal: 1,
        };
        let values = volume_oscillator_store(&store, params, &mut HashMap::new());

        // EMA(2) starts at bar 1 with 15, then 25; EMA(3) starts at bar 2 with 20.
        assert_series_close(&values, &[f64::NAN, f64::NAN, 5.0]);
        assert_eq!(latest_volume_oscillator_store(&store, params), Some(5.0));
    }
}

use crate::indicators::ema::ema_sma_seeded_close_store;
use crate::types::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Price Oscillator in points: fast EMA - slow EMA of the close. The EMAs start with a simple
/// average, so the first value is at bar slow - 1. PPO gives the same gap as a percentage.
pub fn price_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("price_osc_pts:close:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let fast_ema = ema_sma_seeded_close_store(store, params.fast, nodes);
    let slow_ema = ema_sma_seeded_close_store(store, params.slow, nodes);
    let out: Vec<f64> = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(f, s)| f - s)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_price_oscillator_store(store: &CandleStore, params: MacdParams) -> Option<f64> {
    price_oscillator_store(store, params, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "PRICE_OSCILLATOR",
        name: "PRICE OSCILLATOR",
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
    fn price_oscillator_is_zero_when_fast_and_slow_emas_match() {
        let store = close_store(&[5.0, 5.0, 5.0, 5.0]);
        let params = MacdParams {
            fast: 2,
            slow: 3,
            signal: 1,
        };
        let values = price_oscillator_store(&store, params, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 0.0, 0.0]);
        assert_eq!(latest_price_oscillator_store(&store, params), Some(0.0));
    }

    #[test]
    fn price_oscillator_is_the_gap_between_the_averages_in_points() {
        // A straight line: each EMA lags the close by (period - 1) / 2.
        let closes: Vec<f64> = (0..10).map(|i| 100.0 + 2.0 * i as f64).collect();
        let store = close_store(&closes);
        let params = MacdParams {
            fast: 3,
            slow: 5,
            signal: 1,
        };
        let values = price_oscillator_store(&store, params, &mut HashMap::new());
        // Lags of 1 and 2 bars at 2 points a bar (2 and 4 points): the gap is 2 points from bar 4.
        assert!(values[..4].iter().all(|v| v.is_nan()));
        assert!(values[4..].iter().all(|v| (v - 2.0).abs() < 1e-9));
    }
}

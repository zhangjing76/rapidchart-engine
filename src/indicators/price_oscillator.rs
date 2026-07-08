use crate::indicators::ema::ema_close_store;
use crate::types::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Price Oscillator: ((short EMA - long EMA) / long EMA) * 100
/// Similar to PPO but expressed as percentage of the long EMA.
pub fn price_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("price_osc:close:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let fast_ema = ema_close_store(store, params.fast, nodes);
    let slow_ema = ema_close_store(store, params.slow, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let f = fast_ema[i];
        let s = slow_ema[i];
        if !f.is_nan() && !s.is_nan() && s.abs() > 1e-10 {
            out[i] = ((f - s) / s) * 100.0;
        }
    }
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
        let store = close_store(&[5.0, 5.0, 5.0]);
        let params = MacdParams {
            fast: 2,
            slow: 3,
            signal: 1,
        };
        let values = price_oscillator_store(&store, params, &mut HashMap::new());

        assert_series_close(&values, &[0.0, 0.0, 0.0]);
        assert_eq!(latest_price_oscillator_store(&store, params), Some(0.0));
    }
}

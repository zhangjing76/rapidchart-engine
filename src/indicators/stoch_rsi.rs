use crate::indicators::rsi::rsi_close_store;
use crate::indicators::stoch::{smooth_series, stochastic_k_values};
use crate::value_at_slice;
use crate::CandleStore;
use crate::NodeCache;
use std::collections::HashMap;
use std::rc::Rc;

pub fn stoch_rsi_store(
    store: &CandleStore,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let rsi = rsi_close_store(store, period, nodes);
    let raw_k = stochastic_k_values(&rsi, stoch_period);
    let k = smooth_series(&raw_k, smooth);
    let d = smooth_series(&k, signal);
    let outputs = vec![crate::named_series("k", k), crate::named_series("d", d)];
    nodes.insert(
        format!("stoch:rsi:{period}:{stoch_period}:{smooth}:{signal}"),
        Rc::clone(&outputs[0].values),
    );
    outputs
}
pub fn latest_stoch_rsi_store(
    store: &CandleStore,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
) -> (Option<f64>, Option<f64>) {
    let outputs = stoch_rsi_store(
        store,
        period,
        stoch_period,
        smooth,
        signal,
        &mut HashMap::new(),
    );
    let index = store.len().saturating_sub(1);
    (
        value_at_slice(outputs[0].values.as_slice(), index),
        value_at_slice(outputs[1].values.as_slice(), index),
    )
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "STOCH_RSI",
                name: "STOCH RSI",
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
                        name: "stoch_period",
                        label: "Stoch",
                        default: 14.0,
                        min: 1.0,
                        step: "1",
                    },
                    crate::descriptors::ParamDescriptor {
                        name: "smooth",
                        label: "%K",
                        default: 3.0,
                        min: 1.0,
                        step: "1",
                    },
                    crate::descriptors::ParamDescriptor {
                        name: "signal",
                        label: "%D",
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
    fn stoch_rsi_is_the_manual_smoothed_rsi_stochastic() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let outputs = stoch_rsi_store(&store, 3, 3, 2, 2, &mut HashMap::new());

        assert_series_close(
            outputs[0].values.as_slice(),
            &[
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                0.0,
                0.0,
            ],
        );
        assert_series_close(
            outputs[1].values.as_slice(),
            &[
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                0.0,
            ],
        );
        assert_eq!(
            latest_stoch_rsi_store(&store, 3, 3, 2, 2),
            (Some(0.0), Some(0.0))
        );
    }
}

use crate::indicators::atr::{atr_store, latest_atr_store};
use crate::indicators::bollinger::bollinger_outputs;
use crate::indicators::ema::{ema_close_store, latest_ema_store};
use crate::indicators::sma::{latest_sma_store, sma_close_store};
use crate::rc_into_owned;
use crate::CandleStore;
use crate::IndicatorArena;
use crate::NodeCache;
use std::rc::Rc;

const MIDDLE_SLOT: usize = 1;
const ATR_STATE_SLOT: usize = 3;

pub fn keltner_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let middle = rc_into_owned(ema_close_store(store, period, nodes));
    let atr = rc_into_owned(atr_store(store, period, nodes));
    let mut upper = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    for ((upper_val, lower_val), (&mid, &atr_value)) in upper
        .iter_mut()
        .zip(lower.iter_mut())
        .zip(middle.iter().zip(atr.iter()))
    {
        if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        *upper_val = mid + multiplier * atr_value;
        *lower_val = mid - multiplier * atr_value;
    }
    let outputs = vec![
        crate::named_series("upper", upper),
        crate::named_series("middle", middle),
        crate::named_series("lower", lower),
        crate::named_series("atr_state", atr),
    ];
    for output in &outputs {
        nodes.insert(
            format!("keltner:{}:{}:{}", output.name, period, multiplier),
            Rc::clone(&output.values),
        );
    }
    outputs
}
pub fn latest_keltner_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_ema_store(store, period, outputs.get_slot(MIDDLE_SLOT));
    let atr = latest_atr_store(store, period, outputs.get_slot(ATR_STATE_SLOT));
    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
}
pub fn starc_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let middle = rc_into_owned(sma_close_store(store, period, nodes));
    let atr = rc_into_owned(atr_store(store, period, nodes));
    let mut upper = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    for ((upper_val, lower_val), (&mid, &atr_value)) in upper
        .iter_mut()
        .zip(lower.iter_mut())
        .zip(middle.iter().zip(atr.iter()))
    {
        if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        *upper_val = mid + multiplier * atr_value;
        *lower_val = mid - multiplier * atr_value;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("starc:{}:{}:{}", output.name, period, multiplier),
            Rc::clone(&output.values),
        );
    }
    outputs
}
pub fn latest_starc_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_sma_store(store, period);
    let atr = latest_atr_store(store, period, None);
    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
}

pub(crate) fn starc_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "STARC",
                name: "STARC",
                category: "Volatility",
                pane: "overlay",
                params: vec![
                    crate::descriptors::ParamDescriptor {
                        name: "period",
                        label: "Period",
                        default: 15.0,
                        min: 1.0,
                        step: "1",
                    },
                    crate::descriptors::ParamDescriptor {
                        name: "multiplier",
                        label: "Multiplier",
                        default: 2.0,
                        min: 0.1,
                        step: "0.1",
                    },
                ],
                outputs: vec![
                    crate::descriptors::output_descriptor("upper", "line", "overlay", "#0f766e"),
                    crate::descriptors::output_descriptor("middle", "line", "overlay", "#2563eb"),
                    crate::descriptors::output_descriptor("lower", "line", "overlay", "#0f766e"),
                ],
            }
}

pub(crate) fn keltner_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "KELTNER",
                name: "KELTNER",
                category: "Averages/Bands",
                pane: "overlay",
                params: vec![
                    crate::descriptors::ParamDescriptor {
                        name: "period",
                        label: "Period",
                        default: 20.0,
                        min: 1.0,
                        step: "1",
                    },
                    crate::descriptors::ParamDescriptor {
                        name: "multiplier",
                        label: "Multiplier",
                        default: 2.0,
                        min: 1.0,
                        step: "0.1",
                    },
                ],
                outputs: vec![
                    crate::descriptors::output_descriptor("upper", "line", "overlay", "#0f766e"),
                    crate::descriptors::output_descriptor("middle", "line", "overlay", "#2563eb"),
                    crate::descriptors::output_descriptor("lower", "line", "overlay", "#0f766e"),
                ],
            }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
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
    fn keltner_is_the_manual_ema_plus_atr_band() {
        let store = ohlc_store(&[
            (1.0, 1.0, 1.0),
            (2.0, 2.0, 2.0),
            (3.0, 3.0, 3.0),
            (4.0, 4.0, 4.0),
            (5.0, 5.0, 5.0),
        ]);
        let outputs = keltner_store(&store, 3, 2.0, &mut HashMap::new());
        let arena = crate::IndicatorArena::from_named_outputs(outputs.clone());

        assert_series_close(
            outputs[0].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 5.125, 6.0625],
        );
        assert_series_close(
            outputs[1].values.as_slice(),
            &[1.0, 1.5, 2.25, 3.125, 4.0625],
        );
        assert_series_close(
            outputs[2].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 1.125, 2.0625],
        );
        assert_series_close(
            outputs[3].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 1.0, 1.0],
        );
        assert_eq!(
            latest_keltner_store(&store, 3, 2.0, &arena),
            (Some(6.0625), Some(4.0625), Some(2.0625))
        );
    }
}

use crate::indicators::atr::{atr_store, latest_atr_store};
use crate::indicators::bollinger::bollinger_outputs;
use crate::indicators::ema::{ema_close_store, latest_ema_store};
use crate::indicators::sma::{latest_sma_store, sma_close_store};
use crate::rc_into_owned;
use crate::IndicatorArena;
use crate::IndicatorOutput;
use crate::NodeCache;
use crate::CandleStore;
use std::rc::Rc;

pub fn keltner_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
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
        IndicatorOutput {
            name: "upper".to_string(),
            values: upper,
        },
        IndicatorOutput {
            name: "middle".to_string(),
            values: middle,
        },
        IndicatorOutput {
            name: "lower".to_string(),
            values: lower,
        },
    ];
    for output in &outputs {
        nodes.insert(
            format!("keltner:{}:{}:{}", output.name, period, multiplier),
            Rc::new(output.values.clone()),
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
    let middle = latest_ema_store(store, period, outputs.get("middle"));
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
pub fn starc_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
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
            Rc::new(output.values.clone()),
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

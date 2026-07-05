use crate::indicators::atr::{atr_node, atr_store, latest_atr, latest_atr_store, true_range_store};
use crate::indicators::bollinger::bollinger_outputs;
use crate::indicators::ema::{ema_close, ema_close_store, latest_ema, latest_ema_store};
use crate::indicators::sma::{latest_sma, latest_sma_store, sma_close, sma_close_store};
use crate::IndicatorArena;
use crate::IndicatorOutput;
use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn keltner(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = ema_close(bars, period, nodes);
    let atr = atr_node(bars, period, nodes);
    let mut upper = vec![f64::NAN; bars.len()];
    let mut lower = vec![f64::NAN; bars.len()];
    for index in 0..bars.len() {
        let mid = middle[index];
        let atr_value = atr[index];
        if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        upper[index] = mid + multiplier * atr_value;
        lower[index] = mid - multiplier * atr_value;
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
    for index in 0..store.len() {
        let mid = middle[index];
        let atr_value = atr[index];
        if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        upper[index] = mid + multiplier * atr_value;
        lower[index] = mid - multiplier * atr_value;
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
pub fn latest_keltner(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_ema(bars, period, outputs.get("middle"));
    let atr = latest_atr(bars, period, None);
    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
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
pub fn starc(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = sma_close(bars, period, nodes);
    let atr = atr_node(bars, period, nodes);
    let mut upper = vec![f64::NAN; bars.len()];
    let mut lower = vec![f64::NAN; bars.len()];
    for index in 0..bars.len() {
        let mid = middle[index];
        let atr_value = atr[index];
        if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        upper[index] = mid + multiplier * atr_value;
        lower[index] = mid - multiplier * atr_value;
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
    for index in 0..store.len() {
        let mid = middle[index];
        let atr_value = atr[index];
        if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        upper[index] = mid + multiplier * atr_value;
        lower[index] = mid - multiplier * atr_value;
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
pub fn latest_starc(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_sma(bars, period);
    let atr = latest_atr(bars, period, None);
    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
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

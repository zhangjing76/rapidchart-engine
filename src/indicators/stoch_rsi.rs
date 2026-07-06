use crate::indicators::rsi::{rsi_close, rsi_close_store};
use crate::indicators::stoch::{smooth_series, stochastic_k_values};
use crate::output_at_vec;
use crate::IndicatorOutput;
use crate::NodeCache;
use crate::{Bar, CandleStore};
use std::collections::HashMap;
use std::rc::Rc;

#[allow(dead_code)]
pub fn stoch_rsi(
    bars: &[Bar],
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let rsi = rsi_close(bars, period, nodes);
    let raw_k = stochastic_k_values(&rsi, stoch_period);
    let k = smooth_series(&raw_k, smooth);
    let d = smooth_series(&k, signal);
    let outputs = vec![
        IndicatorOutput {
            name: "k".to_string(),
            values: k,
        },
        IndicatorOutput {
            name: "d".to_string(),
            values: d,
        },
    ];
    nodes.insert(
        format!("stoch:rsi:{period}:{stoch_period}:{smooth}:{signal}"),
        Rc::new(outputs[0].values.clone()),
    );
    outputs
}
pub fn stoch_rsi_store(
    store: &CandleStore,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let rsi = rsi_close_store(store, period, nodes);
    let raw_k = stochastic_k_values(&rsi, stoch_period);
    let k = smooth_series(&raw_k, smooth);
    let d = smooth_series(&k, signal);
    let outputs = vec![
        IndicatorOutput {
            name: "k".to_string(),
            values: k,
        },
        IndicatorOutput {
            name: "d".to_string(),
            values: d,
        },
    ];
    nodes.insert(
        format!("stoch:rsi:{period}:{stoch_period}:{smooth}:{signal}"),
        Rc::new(outputs[0].values.clone()),
    );
    outputs
}
#[allow(dead_code)]
pub fn latest_stoch_rsi(
    bars: &[Bar],
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
) -> (Option<f64>, Option<f64>) {
    let outputs = stoch_rsi(
        bars,
        period,
        stoch_period,
        smooth,
        signal,
        &mut HashMap::new(),
    );
    let index = bars.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "k", index),
        output_at_vec(&outputs, "d", index),
    )
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
        output_at_vec(&outputs, "k", index),
        output_at_vec(&outputs, "d", index),
    )
}
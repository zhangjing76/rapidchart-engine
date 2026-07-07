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

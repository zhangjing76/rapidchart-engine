use crate::indicators::bollinger::bollinger_outputs;
use crate::CandleStore;
use crate::NodeCache;
use std::rc::Rc;

pub fn donchian_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let mut upper = vec![f64::NAN; store.len()];
    let mut middle = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return bollinger_outputs(upper, middle, lower);
    }
    for index in period - 1..store.len() {
        let high = store.high[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let low = store.low[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        upper[index] = high;
        middle[index] = (high + low) / 2.0;
        lower[index] = low;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("donchian:{}:{}", output.name, period),
            Rc::clone(&output.values),
        );
    }
    outputs
}
pub fn latest_donchian_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let high = store.high[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let low = store.low[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    (Some(high), Some((high + low) / 2.0), Some(low))
}
pub fn price_channel_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let mut upper = vec![f64::NAN; store.len()];
    let mut middle = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return bollinger_outputs(upper, middle, lower);
    }
    for index in period - 1..store.len() {
        let high = store.high[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let low = store.low[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        upper[index] = high;
        middle[index] = (high + low) / 2.0;
        lower[index] = low;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("price_channel:{}:{}", output.name, period),
            Rc::clone(&output.values),
        );
    }
    outputs
}
pub fn latest_price_channel_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let high = store.high[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let low = store.low[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    (Some(high), Some((high + low) / 2.0), Some(low))
}

use crate::indicators::bollinger::bollinger_outputs;
use crate::IndicatorOutput;
use crate::NodeCache;
use crate::{Bar, CandleStore};
use std::collections::HashMap;
use std::rc::Rc;

pub fn donchian(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let mut upper = vec![f64::NAN; bars.len()];
    let mut middle = vec![f64::NAN; bars.len()];
    let mut lower = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return bollinger_outputs(upper, middle, lower);
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let high = window
            .iter()
            .map(|bar| bar.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let low = window
            .iter()
            .map(|bar| bar.low)
            .fold(f64::INFINITY, f64::min);
        upper[index] = high;
        middle[index] = (high + low) / 2.0;
        lower[index] = low;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("donchian:{}:{}", output.name, period),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}
pub fn donchian_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
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
            Rc::new(output.values.clone()),
        );
    }
    outputs
}
#[allow(dead_code)]
pub fn latest_donchian(bars: &[Bar], period: usize) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || bars.len() < period {
        return (None, None, None);
    }
    let window = &bars[bars.len() - period..];
    let high = window
        .iter()
        .map(|bar| bar.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = window
        .iter()
        .map(|bar| bar.low)
        .fold(f64::INFINITY, f64::min);
    (Some(high), Some((high + low) / 2.0), Some(low))
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
pub fn price_channel(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let outputs = donchian(bars, period, &mut HashMap::new());
    for output in &outputs {
        nodes.insert(
            format!("price_channel:{}:{}", output.name, period),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}
pub fn price_channel_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
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
            Rc::new(output.values.clone()),
        );
    }
    outputs
}
#[allow(dead_code)]
pub fn latest_price_channel(
    bars: &[Bar],
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    latest_donchian(bars, period)
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

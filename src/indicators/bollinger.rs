use crate::indicators::sma::sma_close_store;
use crate::rc_into_owned;
use crate::IndicatorOutput;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::rc::Rc;

pub fn bollinger_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let mut upper = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    let middle = rc_into_owned(sma_close_store(store, period, nodes));
    if period == 0 {
        return bollinger_outputs(upper, middle, lower);
    }
    for index in period - 1..store.len() {
        let mean = middle[index];
        if mean.is_nan() {
            continue;
        };
        let variance = store.close[index + 1 - period..=index]
            .iter()
            .map(|close| {
                let diff = close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        let band = variance.sqrt() * multiplier;
        upper[index] = mean + band;
        lower[index] = mean - band;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("bb:{}:{}:{}", output.name, period, multiplier),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}
pub fn bollinger_outputs(upper: Series, middle: Series, lower: Series) -> Vec<IndicatorOutput> {
    vec![
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
    ]
}
pub fn latest_bollinger_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let window = &store.close[store.len() - period..];
    let mean = window.iter().sum::<f64>() / period as f64;
    let variance = window
        .iter()
        .map(|close| {
            let diff = close - mean;
            diff * diff
        })
        .sum::<f64>()
        / period as f64;
    let band = variance.sqrt() * multiplier;
    (Some(mean + band), Some(mean), Some(mean - band))
}

use crate::NodeCache;
use crate::{CandleStore, IndicatorOutput, Series};

/// Rainbow Moving Average: 10 recursive SMAs.
/// Layer 1 = SMA(close, period)
/// Layer 2 = SMA(Layer 1, period)
/// ... Layer 10 = SMA(Layer 9, period)
///
/// Each layer is an SMA of the previous layer's output.
fn sma_of_series(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    // Build running sum skipping NaNs properly
    let mut valid_start = 0;
    // Find first index where we have `period` consecutive non-NaN values
    let mut count = 0;
    for (i, &v) in values.iter().enumerate() {
        if v.is_nan() {
            count = 0;
        } else {
            count += 1;
            if count == period {
                valid_start = i + 1 - period;
                break;
            }
        }
    }
    if count < period {
        return out;
    }
    let start_idx = valid_start + period - 1;
    let mut sum: f64 = values[valid_start..=start_idx].iter().sum();
    out[start_idx] = sum / period as f64;
    for i in start_idx + 1..values.len() {
        if values[i].is_nan() {
            out[i] = f64::NAN;
        } else {
            sum += values[i] - values[i - period];
            out[i] = sum / period as f64;
        }
    }
    out
}


pub fn rainbow_ma_store(
    store: &CandleStore,
    period: usize,
    _nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let mut layers: Vec<Series> = Vec::with_capacity(10);
    let first = sma_of_series(&store.close, period);
    layers.push(first);
    for i in 1..10 {
        let prev = &layers[i - 1];
        layers.push(sma_of_series(prev, period));
    }
    layers
        .into_iter()
        .enumerate()
        .map(|(i, values)| IndicatorOutput {
            name: format!("r{}", i + 1),
            values,
        })
        .collect()
}

pub fn latest_rainbow_ma_store(store: &CandleStore, period: usize) -> Vec<(String, Option<f64>)> {
    let mut layers: Vec<Series> = Vec::with_capacity(10);
    let first = sma_of_series(&store.close, period);
    layers.push(first);
    for i in 1..10 {
        let prev = &layers[i - 1];
        layers.push(sma_of_series(prev, period));
    }
    layers
        .iter()
        .enumerate()
        .map(|(i, values)| {
            let last = values.last().copied().and_then(|v| {
                if v.is_nan() { None } else { Some(v) }
            });
            (format!("r{}", i + 1), last)
        })
        .collect()
}
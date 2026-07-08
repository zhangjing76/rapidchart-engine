use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// SMA of a series
fn sma_of(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    let mut sum: f64 = values[..period].iter().sum();
    out[period - 1] = sum / period as f64;
    for i in period..values.len() {
        sum += values[i] - values[i - period];
        out[i] = sum / period as f64;
    }
    out
}

/// Rainbow Oscillator: (close - average of 10 rainbow MAs) / (highest - lowest rainbow) * 100
/// Uses 10 nested SMAs from rainbow_ma concept.
pub fn rainbow_oscillator_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("rainbow_osc:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // Build 10 rainbow layers
    let mut layers: Vec<Series> = Vec::with_capacity(10);
    layers.push(sma_of(&store.close, period));
    for i in 1..10 {
        layers.push(sma_of(&layers[i - 1], period));
    }
    for i in 0..len {
        let vals: Vec<f64> = layers
            .iter()
            .map(|l| l[i])
            .filter(|v| !v.is_nan())
            .collect();
        if vals.is_empty() {
            continue;
        }
        let avg = vals.iter().sum::<f64>() / vals.len() as f64;
        let max = vals.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min = vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = max - min;
        if range > 1e-10 {
            out[i] = ((store.close[i] - avg) / range) * 100.0;
        } else {
            out[i] = 0.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_rainbow_oscillator_store(store: &CandleStore, period: usize) -> Option<f64> {
    rainbow_oscillator_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
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
    fn rainbow_oscillator_is_zero_for_constant_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let values = rainbow_oscillator_store(&store, 1, &mut HashMap::new());

        assert_eq!(&*values, &[0.0, 0.0, 0.0, 0.0]);
        assert_eq!(latest_rainbow_oscillator_store(&store, 1), Some(0.0));
    }

    #[test]
    fn rainbow_oscillator_currently_stays_zero_on_rising_prices() {
        let store = close_store(&[10.0, 12.0, 14.0, 16.0, 18.0, 20.0]);
        let values = rainbow_oscillator_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(latest_rainbow_oscillator_store(&store, 2), Some(0.0));
    }
}

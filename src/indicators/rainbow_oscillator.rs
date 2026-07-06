use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// SMA of a series
fn sma_of(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period { return out; }
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
pub fn rainbow_oscillator_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
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
        let vals: Vec<f64> = layers.iter().map(|l| l[i]).filter(|v| !v.is_nan()).collect();
        if vals.is_empty() { continue; }
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

pub fn rainbow_oscillator_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("rainbow_osc:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    let close: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let mut layers: Vec<Series> = Vec::with_capacity(10);
    layers.push(sma_of(&close, period));
    for i in 1..10 {
        layers.push(sma_of(&layers[i - 1], period));
    }
    for i in 0..len {
        let vals: Vec<f64> = layers.iter().map(|l| l[i]).filter(|v| !v.is_nan()).collect();
        if vals.is_empty() { continue; }
        let avg = vals.iter().sum::<f64>() / vals.len() as f64;
        let max = vals.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min = vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = max - min;
        if range > 1e-10 {
            out[i] = ((bars[i].close - avg) / range) * 100.0;
        } else {
            out[i] = 0.0;
        }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_rainbow_oscillator_store(store: &CandleStore, period: usize) -> Option<f64> {
    rainbow_oscillator_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

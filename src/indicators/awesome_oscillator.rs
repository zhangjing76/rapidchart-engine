use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Awesome Oscillator: SMA(5) of midpoint - SMA(34) of midpoint
/// where midpoint = (high + low) / 2
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

pub fn awesome_oscillator_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "ao:hl".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let midpoints: Vec<f64> = store.high.iter().zip(store.low.iter())
        .map(|(h, l)| (h + l) / 2.0).collect();
    let sma5 = sma_of(&midpoints, 5);
    let sma34 = sma_of(&midpoints, 34);
    let out: Vec<f64> = sma5.iter().zip(sma34.iter())
        .map(|(a, b)| if a.is_nan() || b.is_nan() { f64::NAN } else { a - b })
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn awesome_oscillator_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "ao:hl".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let midpoints: Vec<f64> = bars.iter().map(|b| (b.high + b.low) / 2.0).collect();
    let sma5 = sma_of(&midpoints, 5);
    let sma34 = sma_of(&midpoints, 34);
    let out: Vec<f64> = sma5.iter().zip(sma34.iter())
        .map(|(a, b)| if a.is_nan() || b.is_nan() { f64::NAN } else { a - b })
        .collect();
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_awesome_oscillator_store(store: &CandleStore) -> Option<f64> {
    awesome_oscillator_store(store, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

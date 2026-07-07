use crate::output_at;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::rc::Rc;

pub fn stochastic_k_values(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    for index in period - 1..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        let highest = window.iter().copied().fold(f64::MIN, f64::max);
        let lowest = window.iter().copied().fold(f64::MAX, f64::min);
        let range = highest - lowest;
        let current = values[index];
        out[index] = if range == 0.0 {
            0.0
        } else {
            100.0 * (current - lowest) / range
        };
    }
    out
}
pub fn smooth_series(values: &[f64], smooth: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if smooth == 0 {
        return out;
    }
    for (index, out_val) in out.iter_mut().enumerate() {
        if index + 1 < smooth {
            continue;
        }
        let window = &values[index + 1 - smooth..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        *out_val = window.iter().sum::<f64>() / smooth as f64;
    }
    out
}
pub fn stochastic_k_store(store: &CandleStore, period: usize) -> Series {
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return out;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
        let window = index + 1 - period..=index;
        let highest_high = window
            .clone()
            .map(|i| store.high[i])
            .fold(f64::MIN, f64::max);
        let lowest_low = window.map(|i| store.low[i]).fold(f64::MAX, f64::min);
        let range = highest_high - lowest_low;
        *item = if range == 0.0 {
            0.0
        } else {
            100.0 * (store.close[index] - lowest_low) / range
        };
    }
    out
}
pub fn stochastic_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let k = stochastic_k_store(store, period);
    let d = smooth_series(&k, smooth);
    let outputs = vec![
        crate::named_series("k", k,),
        crate::named_series("d", d,),
    ];
    nodes.insert(
        format!("stoch:hlc:{period}:{smooth}"),
        Rc::clone(&outputs[0].values),
    );
    outputs
}
pub fn latest_stochastic_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None);
    }
    let start = store.len() - period;
    let highest_high = store.high[start..].iter().copied().fold(f64::MIN, f64::max);
    let lowest_low = store.low[start..].iter().copied().fold(f64::MAX, f64::min);
    let range = highest_high - lowest_low;
    let k = if range == 0.0 {
        0.0
    } else {
        100.0 * (store.close[store.len() - 1] - lowest_low) / range
    };
    if smooth == 0 || store.len() < period + smooth - 1 {
        return (Some(k), None);
    }
    let mut values = Vec::with_capacity(smooth);
    for index in store.len() - smooth..store.len() - 1 {
        let Some(value) = output_at(outputs, "k", index) else {
            return (Some(k), None);
        };
        values.push(value);
    }
    values.push(k);
    (Some(k), Some(values.iter().sum::<f64>() / smooth as f64))
}

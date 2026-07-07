use crate::indicators::ema::ema_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Schaff Trend Cycle: MACD line passed through double Stochastic smoothing.
/// 1. Compute MACD line = EMA(fast) - EMA(slow)
/// 2. Apply Stochastic %K to MACD over stoch_period
/// 3. Smooth with EMA (factor 0.5)
/// 4. Apply Stochastic %K again
/// 5. Smooth with EMA (factor 0.5) → STC
pub fn schaff_trend_cycle_store(
    store: &CandleStore,
    fast: usize,
    slow: usize,
    stoch_period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("stc:{}:{}:{}", fast, slow, stoch_period);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema_fast = ema_close_store(store, fast, nodes);
    let ema_slow = ema_close_store(store, slow, nodes);
    let len = store.len();
    // MACD line
    let mut macd_line = vec![f64::NAN; len];
    for i in 0..len {
        if !ema_fast[i].is_nan() && !ema_slow[i].is_nan() {
            macd_line[i] = ema_fast[i] - ema_slow[i];
        }
    }
    // First stochastic of MACD
    let stoch1 = stochastic_of(&macd_line, stoch_period);
    // EMA smooth with factor 0.5
    let smooth1 = ema_factor(&stoch1, 0.5);
    // Second stochastic
    let stoch2 = stochastic_of(&smooth1, stoch_period);
    // EMA smooth with factor 0.5
    let out = ema_factor(&stoch2, 0.5);
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

/// Rolling stochastic: (value - min) / (max - min) * 100
fn stochastic_of(values: &[f64], period: usize) -> Series {
    let len = values.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 {
        return out;
    }
    for i in period - 1..len {
        let window = &values[i + 1 - period..=i];
        let valid: Vec<f64> = window.iter().filter(|v| !v.is_nan()).copied().collect();
        if valid.is_empty() {
            continue;
        }
        let max = valid.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min = valid.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = max - min;
        let v = values[i];
        if !v.is_nan() && range > 1e-10 {
            out[i] = ((v - min) / range) * 100.0;
        } else if !v.is_nan() {
            out[i] = 50.0;
        }
    }
    out
}

/// EMA with a fixed factor (not period-based)
fn ema_factor(values: &[f64], factor: f64) -> Series {
    let mut out = Vec::with_capacity(values.len());
    let mut prev = None::<f64>;
    for &v in values {
        if v.is_nan() {
            out.push(f64::NAN);
            continue;
        }
        let next = match prev {
            Some(p) => factor * v + (1.0 - factor) * p,
            None => v,
        };
        prev = Some(next);
        out.push(next);
    }
    out
}

pub fn latest_schaff_trend_cycle_store(
    store: &CandleStore,
    fast: usize,
    slow: usize,
    stoch_period: usize,
) -> Option<f64> {
    schaff_trend_cycle_store(store, fast, slow, stoch_period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

use crate::indicators::adl::adl_store;
use crate::indicators::ema::ema_series;
use crate::IndicatorArena;
use crate::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn chaikin_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("chaikin:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let adl_values = adl_store(store, nodes);
    let fast = ema_series(&adl_values, params.fast);
    let slow = ema_series(&adl_values, params.slow);
    let values: Vec<_> = fast
        .iter()
        .zip(slow.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() => *fast - *slow,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn chaikin_volatility_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("cvol:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema_key = format!("cvol:ema:{period}");
    let ranges: Vec<_> = (0..store.len())
        .map(|index| store.high[index] - store.low[index])
        .collect();
    let ema = ema_series(&ranges, period);
    nodes.insert(ema_key, Rc::new(ema.clone()));
    let mut values = vec![f64::NAN; store.len()];
    if period != 0 && store.len() > period {
        for index in period..store.len() {
            match (ema[index], ema[index - period]) {
                (current, previous)
                    if !current.is_nan() && !previous.is_nan() && previous != 0.0 =>
                {
                    values[index] = 100.0 * (current - previous) / previous;
                }
                (current2, previous2) if !current2.is_nan() && !previous2.is_nan() => {
                    values[index] = 0.0
                }
                _ => {}
            }
        }
    }
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_chaikin_volatility_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    if period == 0 || store.len() <= period {
        return (None, None);
    }
    let range = store.high[store.len() - 1] - store.low[store.len() - 1];
    let alpha = 2.0 / (period as f64 + 1.0);
    let prev_ema = outputs
        .get("hl_ema")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(range);
    let hl_ema = alpha * range + (1.0 - alpha) * prev_ema;
    // Need EMA from `period` bars ago
    let period_ago_ema = outputs
        .get("hl_ema")
        .and_then(|s| s.get(store.len() - 1 - period).copied())
        .filter(|v| !v.is_nan());
    let value = match period_ago_ema {
        Some(prev) if prev != 0.0 => Some(100.0 * (hl_ema - prev) / prev),
        Some(_) => Some(0.0),
        None => None,
    };
    (value, Some(hl_ema))
}

pub fn latest_chaikin_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if store.len() == 0 {
        return (None, None, None, None);
    }
    // Compute latest ADL value
    let i = store.len() - 1;
    let range = store.high[i] - store.low[i];
    let mfm = if range == 0.0 {
        0.0
    } else {
        ((store.close[i] - store.low[i]) - (store.high[i] - store.close[i])) / range
    };
    let mfv = mfm * store.volume[i];
    let prev_adl = outputs
        .get("adl")
        .and_then(|s| s.get(store.len().wrapping_sub(2)).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(0.0);
    let adl = if store.len() == 1 {
        mfv
    } else {
        prev_adl + mfv
    };
    // EMA steps
    let alpha_fast = 2.0 / (params.fast as f64 + 1.0);
    let alpha_slow = 2.0 / (params.slow as f64 + 1.0);
    let prev_fast = outputs
        .get("fast_ema")
        .and_then(|s| s.get(store.len().wrapping_sub(2)).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(adl);
    let fast_ema = if store.len() == 1 {
        adl
    } else {
        alpha_fast * adl + (1.0 - alpha_fast) * prev_fast
    };
    let prev_slow = outputs
        .get("slow_ema")
        .and_then(|s| s.get(store.len().wrapping_sub(2)).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(adl);
    let slow_ema = if store.len() == 1 {
        adl
    } else {
        alpha_slow * adl + (1.0 - alpha_slow) * prev_slow
    };
    (
        Some(fast_ema - slow_ema),
        Some(adl),
        Some(fast_ema),
        Some(slow_ema),
    )
}

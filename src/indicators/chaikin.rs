use crate::indicators::adl::adl_store;
use crate::indicators::ema::ema_series;
use crate::nan_to_none;
use crate::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

pub fn chaikin_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Series {
    let key = format!("chaikin:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
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
    nodes.insert(key, Rc::new(values.clone()));
    values
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
pub fn latest_chaikin_volatility_store(store: &CandleStore, period: usize) -> Option<f64> {
    chaikin_volatility_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}
pub fn latest_chaikin_oscillator_store(store: &CandleStore, params: MacdParams) -> Option<f64> {
    chaikin_oscillator_store(store, params, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}
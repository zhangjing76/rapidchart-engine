use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use crate::indicators::ema::ema_close_store;
use crate::types::MacdParams;
use std::collections::HashMap;
use std::rc::Rc;

/// Price Oscillator: ((short EMA - long EMA) / long EMA) * 100
/// Similar to PPO but expressed as percentage of the long EMA.
pub fn price_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("price_osc:close:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let fast_ema = ema_close_store(store, params.fast, nodes);
    let slow_ema = ema_close_store(store, params.slow, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let f = fast_ema[i];
        let s = slow_ema[i];
        if !f.is_nan() && !s.is_nan() && s.abs() > 1e-10 {
            out[i] = ((f - s) / s) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_price_oscillator_store(store: &CandleStore, params: MacdParams) -> Option<f64> {
    price_oscillator_store(store, params, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}
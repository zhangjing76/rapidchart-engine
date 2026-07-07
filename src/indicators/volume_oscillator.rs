use crate::indicators::ema::ema_series;
use crate::types::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Volume Oscillator: ((short vol EMA - long vol EMA) / long vol EMA) * 100
pub fn volume_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("vol_osc:volume:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let fast_ema = ema_series(&store.volume, params.fast);
    let slow_ema = ema_series(&store.volume, params.slow);
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

pub fn latest_volume_oscillator_store(store: &CandleStore, params: MacdParams) -> Option<f64> {
    volume_oscillator_store(store, params, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

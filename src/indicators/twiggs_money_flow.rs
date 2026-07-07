use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Twiggs Money Flow: EMA-smoothed version of Chaikin Money Flow.
/// Uses Wilder-style EMA (alpha = 1/period) to smooth ADL and volume.
/// TMF = EMA(ADL_change, period) / EMA(volume, period)
/// where ADL_change = ((close - true_low) - (true_high - close)) / (true_high - true_low) * volume
/// true_high = max(high, prev_close), true_low = min(low, prev_close)
pub fn twiggs_money_flow_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("tmf:hlcv:{period}");
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 || period == 0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut ema_adl = 0.0f64;
    let mut ema_vol = 0.0f64;
    let mut initialized = false;
    for i in 1..len {
        let true_high = store.high[i].max(store.close[i-1]);
        let true_low = store.low[i].min(store.close[i-1]);
        let range = true_high - true_low;
        let adl_change = if range > 1e-10 {
            ((store.close[i] - true_low) - (true_high - store.close[i])) / range * store.volume[i]
        } else {
            0.0
        };
        if !initialized {
            ema_adl = adl_change;
            ema_vol = store.volume[i];
            initialized = true;
        } else {
            ema_adl = alpha * adl_change + (1.0 - alpha) * ema_adl;
            ema_vol = alpha * store.volume[i] + (1.0 - alpha) * ema_vol;
        }
        if ema_vol.abs() > 1e-10 {
            out[i] = ema_adl / ema_vol;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_twiggs_money_flow_store(store: &CandleStore, period: usize) -> Option<f64> {
    twiggs_money_flow_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}
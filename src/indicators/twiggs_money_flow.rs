use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Twiggs Money Flow: EMA-smoothed version of Chaikin Money Flow.
/// Uses Wilder-style EMA (alpha = 1/period) to smooth ADL and volume.
/// TMF = EMA(ADL_change, period) / EMA(volume, period)
/// where ADL_change = ((close - true_low) - (true_high - close)) / (true_high - true_low) * volume
/// true_high = max(high, prev_close), true_low = min(low, prev_close)
pub fn twiggs_money_flow_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("tmf:hlcv:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
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
        let true_high = store.high[i].max(store.close[i - 1]);
        let true_low = store.low[i].min(store.close[i - 1]);
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
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "TWIGGS_MONEY_FLOW",
        "TWIGGS MONEY FLOW",
        "Volume",
        "separate",
        21,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            vec![1.0; len],
        )
    }

    #[test]
    fn twiggs_money_flow_is_one_when_close_stays_at_the_true_high() {
        let store = ohlcv_store(&[(2.0, 0.0, 2.0), (2.0, 0.0, 2.0), (2.0, 0.0, 2.0)]);
        let values = twiggs_money_flow_store(&store, 2, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert!((values[1] - 1.0).abs() < 1e-12);
        assert!((values[2] - 1.0).abs() < 1e-12);
        assert_eq!(latest_twiggs_money_flow_store(&store, 2), Some(1.0));
    }
}

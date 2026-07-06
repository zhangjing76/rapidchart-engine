use crate::NodeCache;
use crate::{CandleStore, IndicatorOutput};
use crate::indicators::atr::atr_store;
use crate::indicators::ema::ema_close_store;
use crate::series::rc_into_owned;

/// ATR Bands: EMA(close, period) ± multiplier * ATR(period)

pub fn atr_bands_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle_rc = ema_close_store(store, period, nodes);
    let atr_rc = atr_store(store, period, nodes);
    let len = store.len();
    let mut upper = vec![f64::NAN; len];
    let mut lower = vec![f64::NAN; len];
    for i in 0..len {
        let m = middle_rc[i];
        let a = atr_rc[i];
        if !m.is_nan() && !a.is_nan() {
            upper[i] = m + multiplier * a;
            lower[i] = m - multiplier * a;
        }
    }
    let middle = rc_into_owned(middle_rc);
    vec![
        IndicatorOutput { name: "upper".to_string(), values: upper },
        IndicatorOutput { name: "middle".to_string(), values: middle },
        IndicatorOutput { name: "lower".to_string(), values: lower },
    ]
}

pub fn latest_atr_bands_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    outputs: &crate::types::IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    // Reuse existing EMA and ATR latest logic
    let middle = crate::indicators::ema::latest_ema_store(
        store,
        period,
        outputs.get("middle"),
    );
    let atr_val = crate::indicators::atr::latest_atr_store(
        store,
        period,
        outputs.get("atr_state"),
    );
    match (middle, atr_val) {
        (Some(m), Some(a)) => (Some(m + multiplier * a), Some(m), Some(m - multiplier * a)),
        _ => (None, None, None),
    }
}
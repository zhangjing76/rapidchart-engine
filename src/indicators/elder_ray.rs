use crate::indicators::ema::ema_close_store;
use crate::CandleStore;
use crate::NodeCache;

/// Elder Ray Index:
/// Bull Power = High - EMA(close, period)
/// Bear Power = Low - EMA(close, period)

const EMA_STATE_SLOT: usize = 2;

pub fn elder_ray_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let ema = ema_close_store(store, period, nodes);
    let len = store.len();
    let mut bull = vec![f64::NAN; len];
    let mut bear = vec![f64::NAN; len];
    for i in 0..len {
        if !ema[i].is_nan() {
            bull[i] = store.high[i] - ema[i];
            bear[i] = store.low[i] - ema[i];
        }
    }
    vec![
        crate::named_series("bull", bull),
        crate::named_series("bear", bear),
        crate::named_series("ema_state", ema),
    ]
}

pub fn latest_elder_ray_store(
    store: &CandleStore,
    period: usize,
    outputs: &crate::types::IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    let ema_val =
        crate::indicators::ema::latest_ema_store(store, period, outputs.get_slot(EMA_STATE_SLOT));
    match ema_val {
        Some(e) => {
            let i = store.len() - 1;
            (Some(store.high[i] - e), Some(store.low[i] - e))
        }
        None => (None, None),
    }
}

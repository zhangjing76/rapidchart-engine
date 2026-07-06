use crate::NodeCache;
use crate::{Bar, CandleStore, IndicatorOutput};
use crate::indicators::ema::{ema_close, ema_close_store};

/// Elder Ray Index:
/// Bull Power = High - EMA(close, period)
/// Bear Power = Low - EMA(close, period)
pub fn elder_ray(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let ema = ema_close(bars, period, nodes);
    let len = bars.len();
    let mut bull = vec![f64::NAN; len];
    let mut bear = vec![f64::NAN; len];
    for i in 0..len {
        if !ema[i].is_nan() {
            bull[i] = bars[i].high - ema[i];
            bear[i] = bars[i].low - ema[i];
        }
    }
    vec![
        IndicatorOutput { name: "bull".to_string(), values: bull },
        IndicatorOutput { name: "bear".to_string(), values: bear },
    ]
}

pub fn elder_ray_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
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
        IndicatorOutput { name: "bull".to_string(), values: bull },
        IndicatorOutput { name: "bear".to_string(), values: bear },
    ]
}

pub fn latest_elder_ray_store(
    store: &CandleStore,
    period: usize,
    outputs: &crate::types::IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    let ema_val = crate::indicators::ema::latest_ema_store(
        store, period, outputs.get("ema_state"),
    );
    match ema_val {
        Some(e) => {
            let i = store.len() - 1;
            (Some(store.high[i] - e), Some(store.low[i] - e))
        }
        None => (None, None),
    }
}

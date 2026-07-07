use crate::indicators::ema::ema_series;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn force_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("force:close:volume:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut raw = vec![f64::NAN; store.len()];
    for (index, item) in raw.iter_mut().enumerate().skip(1) {
        *item = (store.close[index] - store.close[index - 1]) * store.volume[index];
    }
    let values = ema_series(&raw, period);
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_force_index_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    if store.len() < 2 {
        return (None, None);
    }
    let raw = (store.close[store.len() - 1] - store.close[store.len() - 2])
        * store.volume[store.len() - 1];
    let alpha = 2.0 / (period as f64 + 1.0);
    let prev_ema = outputs
        .get("fi_ema")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(raw);
    let ema = alpha * raw + (1.0 - alpha) * prev_ema;
    (Some(ema), Some(ema))
}

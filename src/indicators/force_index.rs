use crate::indicators::ema::ema_series;
use crate::nan_to_none;
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
pub fn latest_force_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    if store.len() < 2 {
        return None;
    }
    let mut raw = vec![f64::NAN; store.len()];
    for (index, item) in raw.iter_mut().enumerate().skip(1) {
        *item = (store.close[index] - store.close[index - 1]) * store.volume[index];
    }
    ema_series(&raw, period)
        .last()
        .copied()
        .and_then(nan_to_none)
}

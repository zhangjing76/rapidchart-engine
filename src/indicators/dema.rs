use crate::indicators::ema::{ema_close_store, ema_series};
use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

pub fn dema_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("dema:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema1 = rc_into_owned(ema_close_store(store, period, nodes));
    let ema2_key = format!("dema:ema2:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let values: Vec<_> = ema1
        .iter()
        .zip(ema2.iter())
        .map(|(first, second)| match (first, second) {
            (first, second) if !first.is_nan() && !second.is_nan() => 2.0 * *first - *second,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_dema_store(store: &CandleStore, period: usize) -> Option<f64> {
    dema_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

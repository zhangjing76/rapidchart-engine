use crate::indicators::ema::{ema_close_store, ema_series};
use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

pub fn tema_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("tema:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema1 = rc_into_owned(ema_close_store(store, period, nodes));
    let ema2_key = format!("tema:ema2:{period}");
    let ema3_key = format!("tema:ema3:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let ema3 = nodes
        .get(&ema3_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema2, period));
    nodes.insert(ema3_key, Rc::new(ema3.clone()));
    let values: Vec<_> = ema1
        .iter()
        .zip(ema2.iter())
        .zip(ema3.iter())
        .map(|((first, second), third)| match (first, second, third) {
            (first, second, third) if !first.is_nan() && !second.is_nan() && !third.is_nan() => {
                3.0 * *first - 3.0 * *second + *third
            }
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_tema_store(store: &CandleStore, period: usize) -> Option<f64> {
    tema_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

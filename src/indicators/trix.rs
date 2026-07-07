use crate::indicators::ema::{ema_close_store, ema_series};
use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

pub fn trix_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("trix:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema1 = rc_into_owned(ema_close_store(store, period, nodes));
    let ema2_key = format!("trix:ema2:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let ema3 = ema_series(&ema2, period);
    let mut out = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        {
            let previous = ema3[index - 1];
            let current = ema3[index];
            if !previous.is_nan() && !current.is_nan() {
                out[index] = if previous != 0.0 {
                    100.0 * (current / previous - 1.0)
                } else {
                    0.0
                };
            }
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_trix_store(store: &CandleStore, period: usize) -> Option<f64> {
    trix_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

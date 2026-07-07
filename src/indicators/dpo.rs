use crate::indicators::sma::{latest_sma_store, sma_close_store};
use crate::rc_into_owned;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn dpo_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("dpo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma_values = rc_into_owned(sma_close_store(store, period, nodes));
    let shift = period / 2 + 1;
    let mut out = vec![f64::NAN; store.len()];
    for (index, (out_val, &mean)) in out.iter_mut().zip(sma_values.iter()).enumerate() {
        if index < period.saturating_sub(1) || index < shift {
            continue;
        }
        if !mean.is_nan() {
            *out_val = store.close[index - shift] - mean;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_dpo_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let shift = period / 2 + 1;
    let index = store.len() - 1;
    if index < shift || index < period.saturating_sub(1) {
        return None;
    }
    latest_sma_store(store, period).map(|mean| store.close[index - shift] - mean)
}

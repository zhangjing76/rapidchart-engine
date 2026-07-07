use crate::indicators::kst::sma_from_series;
use crate::indicators::sma::sma_close_store;
use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

pub fn trima_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("trima:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let values = sma_from_series(&sma_close_store(store, period, nodes), period);
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_trima_store(store: &CandleStore, period: usize) -> Option<f64> {
    trima_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

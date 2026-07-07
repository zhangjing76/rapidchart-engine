use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

pub fn bop_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "bop:ohlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let values: Vec<_> = (0..store.len())
        .map(|index| {
            let range = store.high[index] - store.low[index];
            if range == 0.0 {
                0.0
            } else {
                (store.close[index] - store.open[index]) / range
            }
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_bop_store(store: &CandleStore) -> Option<f64> {
    bop_store(store, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

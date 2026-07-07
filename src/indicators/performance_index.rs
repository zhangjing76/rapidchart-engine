use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Performance Index: cumulative percentage return from first bar, normalized to 100.
/// value[i] = (close[i] / close[0]) * 100
pub fn performance_index_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "perf_index:close".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len == 0 || store.close[0] == 0.0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let base = store.close[0];
    for i in 0..len {
        out[i] = (store.close[i] / base) * 100.0;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_performance_index_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 || store.close[0] == 0.0 {
        return None;
    }
    let i = store.len() - 1;
    Some((store.close[i] / store.close[0]) * 100.0)
}
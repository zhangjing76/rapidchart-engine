use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

pub fn momentum_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("momentum:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if store.len() <= period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period) {
        *item = store.close[index] - store.close[index - period];
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_momentum_store(store: &CandleStore, period: usize) -> Option<f64> {
    momentum_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

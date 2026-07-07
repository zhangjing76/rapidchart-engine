use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Volume Chart: outputs volume as a line series.
pub fn volume_chart_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "vol_chart:v".to_string();
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let rc = Rc::new(store.volume.clone());
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_volume_chart_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 { return None; }
    Some(store.volume[store.len() - 1])
}
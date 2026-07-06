use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Volume Chart: outputs volume as a line series.
pub fn volume_chart_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "vol_chart:v".to_string();
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let rc = Rc::new(store.volume.clone());
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn volume_chart_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "vol_chart:v".to_string();
    if let Some(v) = nodes.get(&key) { return (**v).clone(); }
    let out: Vec<f64> = bars.iter().map(|b| b.volume).collect();
    nodes.insert(key, Rc::new(out.clone()));
    out
}
pub fn latest_volume_chart_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 { return None; }
    Some(store.volume[store.len() - 1])
}

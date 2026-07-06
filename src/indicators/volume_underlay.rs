use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Volume Underlay: outputs volume value, signed positive for up bars (close >= prev close)
/// and negative for down bars (close < prev close). The sign allows the renderer
/// to color the histogram green/red.
pub fn volume_underlay_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "vol_underlay:cv".to_string();
    if let Some(v) = nodes.get(&key) { return Rc::clone(v); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len == 0 { let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); return rc; }
    out[0] = store.volume[0]; // first bar is neutral/positive
    for i in 1..len {
        if store.close[i] >= store.close[i - 1] {
            out[i] = store.volume[i]; // positive = up bar
        } else {
            out[i] = -store.volume[i]; // negative = down bar
        }
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn volume_underlay_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "vol_underlay:cv".to_string();
    if let Some(v) = nodes.get(&key) { return (**v).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if len == 0 { nodes.insert(key, Rc::new(out.clone())); return out; }
    out[0] = bars[0].volume;
    for i in 1..len {
        if bars[i].close >= bars[i - 1].close {
            out[i] = bars[i].volume;
        } else {
            out[i] = -bars[i].volume;
        }
    }
    nodes.insert(key, Rc::new(out.clone())); out
}
pub fn latest_volume_underlay_store(store: &CandleStore) -> Option<f64> {
    let len = store.len();
    if len == 0 { return None; }
    if len == 1 { return Some(store.volume[0]); }
    let i = len - 1;
    if store.close[i] >= store.close[i - 1] {
        Some(store.volume[i])
    } else {
        Some(-store.volume[i])
    }
}

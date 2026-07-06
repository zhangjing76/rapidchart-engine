use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// ZigZag: connects significant highs and lows based on a percentage threshold.
/// The multiplier parameter is used as the percentage threshold (default 5%).
/// Only changes direction when price moves by at least threshold% from the last pivot.
/// Intermediate values are linearly interpolated.
pub fn zigzag_store(store: &CandleStore, threshold_pct: f64, _nodes: &mut NodeCache) -> RcSeries {
    let key = format!("zigzag:hl:{}", threshold_pct);
    if let Some(v) = _nodes.get(&key) { return Rc::clone(v); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 || threshold_pct <= 0.0 {
        let rc = Rc::new(out); _nodes.insert(key, Rc::clone(&rc)); return rc;
    }
    let threshold = threshold_pct / 100.0;
    // Find pivots
    let mut pivots: Vec<(usize, f64)> = Vec::new();
    let mut trend = 0i8; // 0=unknown, 1=up, -1=down
    let mut last_high = store.high[0];
    let mut last_low = store.low[0];
    let mut last_high_idx = 0usize;
    let mut last_low_idx = 0usize;
    pivots.push((0, store.close[0]));
    for i in 1..len {
        if trend >= 0 {
            if store.high[i] > last_high { last_high = store.high[i]; last_high_idx = i; }
            if store.low[i] < last_high * (1.0 - threshold) {
                // Reversal down
                pivots.push((last_high_idx, last_high));
                trend = -1;
                last_low = store.low[i]; last_low_idx = i;
            }
        }
        if trend <= 0 {
            if store.low[i] < last_low { last_low = store.low[i]; last_low_idx = i; }
            if store.high[i] > last_low * (1.0 + threshold) {
                // Reversal up
                pivots.push((last_low_idx, last_low));
                trend = 1;
                last_high = store.high[i]; last_high_idx = i;
            }
        }
        if trend == 0 {
            if store.high[i] > last_high { last_high = store.high[i]; last_high_idx = i; }
            if store.low[i] < last_low { last_low = store.low[i]; last_low_idx = i; }
            if last_high > store.close[0] * (1.0 + threshold) { trend = 1; }
            else if last_low < store.close[0] * (1.0 - threshold) { trend = -1; }
        }
    }
    // Add final point
    if trend == 1 { pivots.push((last_high_idx, last_high)); }
    else if trend == -1 { pivots.push((last_low_idx, last_low)); }
    else { pivots.push((len - 1, store.close[len - 1])); }
    // Interpolate between pivots
    for w in pivots.windows(2) {
        let (i0, v0) = w[0];
        let (i1, v1) = w[1];
        if i1 == i0 { out[i0] = v0; continue; }
        for j in i0..=i1 {
            let t = (j - i0) as f64 / (i1 - i0) as f64;
            out[j] = v0 + t * (v1 - v0);
        }
    }
    let rc = Rc::new(out); _nodes.insert(key, Rc::clone(&rc)); rc
}
pub fn zigzag_node(bars: &[Bar], threshold_pct: f64, _nodes: &mut NodeCache) -> Series {
    let key = format!("zigzag:hl:{}", threshold_pct);
    if let Some(v) = _nodes.get(&key) { return (**v).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 || threshold_pct <= 0.0 { _nodes.insert(key, Rc::new(out.clone())); return out; }
    let threshold = threshold_pct / 100.0;
    let mut pivots: Vec<(usize, f64)> = Vec::new();
    let mut trend = 0i8;
    let mut last_high = bars[0].high; let mut last_low = bars[0].low;
    let mut last_high_idx = 0; let mut last_low_idx = 0;
    pivots.push((0, bars[0].close));
    for i in 1..len {
        if trend >= 0 {
            if bars[i].high > last_high { last_high = bars[i].high; last_high_idx = i; }
            if bars[i].low < last_high * (1.0 - threshold) {
                pivots.push((last_high_idx, last_high));
                trend = -1; last_low = bars[i].low; last_low_idx = i;
            }
        }
        if trend <= 0 {
            if bars[i].low < last_low { last_low = bars[i].low; last_low_idx = i; }
            if bars[i].high > last_low * (1.0 + threshold) {
                pivots.push((last_low_idx, last_low));
                trend = 1; last_high = bars[i].high; last_high_idx = i;
            }
        }
        if trend == 0 {
            if bars[i].high > last_high { last_high = bars[i].high; last_high_idx = i; }
            if bars[i].low < last_low { last_low = bars[i].low; last_low_idx = i; }
            if last_high > bars[0].close * (1.0 + threshold) { trend = 1; }
            else if last_low < bars[0].close * (1.0 - threshold) { trend = -1; }
        }
    }
    if trend == 1 { pivots.push((last_high_idx, last_high)); }
    else if trend == -1 { pivots.push((last_low_idx, last_low)); }
    else { pivots.push((len - 1, bars[len - 1].close)); }
    for w in pivots.windows(2) {
        let (i0, v0) = w[0]; let (i1, v1) = w[1];
        if i1 == i0 { out[i0] = v0; continue; }
        for j in i0..=i1 { out[j] = v0 + (j - i0) as f64 / (i1 - i0) as f64 * (v1 - v0); }
    }
    _nodes.insert(key, Rc::new(out.clone())); out
}
pub fn latest_zigzag_store(store: &CandleStore, threshold_pct: f64) -> Option<f64> {
    zigzag_store(store, threshold_pct, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

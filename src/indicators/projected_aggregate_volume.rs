use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Projected Aggregate Volume:
/// Projects total session volume based on the pace so far.
/// Uses a rolling window (`period`) as the session length proxy.
/// At bar i within the window:
///   cumulative_volume_so_far / bars_elapsed * total_bars_in_session
///
/// This gives an estimate of what total volume will be by end of the period window,
/// based on the current rate of volume accumulation.
pub fn projected_aggregate_volume_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("pav:v:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len == 0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in 0..len {
        // How far into the current "session" (period window) are we?
        let session_start = if i >= period { i + 1 - period } else { 0 };
        let bars_elapsed = i - session_start + 1;
        let cum_vol: f64 = store.volume[session_start..=i].iter().sum();
        let session_len = period.min(i + 1);
        if bars_elapsed > 0 {
            out[i] = cum_vol / bars_elapsed as f64 * session_len as f64;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_projected_aggregate_volume_store(store: &CandleStore, period: usize) -> Option<f64> {
    if store.len() == 0 || period == 0 {
        return None;
    }
    let i = store.len() - 1;
    let session_start = if i >= period { i + 1 - period } else { 0 };
    let bars_elapsed = i - session_start + 1;
    let cum_vol: f64 = store.volume[session_start..=i].iter().sum();
    let session_len = period.min(i + 1);
    Some(cum_vol / bars_elapsed as f64 * session_len as f64)
}
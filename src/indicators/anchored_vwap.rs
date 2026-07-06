use crate::nan_to_none;
use crate::NodeCache;
use crate::{Bar, CandleStore, IndicatorOutput};

/// Anchored VWAP: VWAP starting from a user-specified bar index (anchor).
/// Before the anchor, values are NaN.
/// anchor = 0 means start from the first bar (equivalent to session VWAP).
pub fn anchored_vwap(bars: &[Bar], anchor: usize, _nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let len = bars.len();
    let mut values = vec![f64::NAN; len];
    if anchor >= len {
        return vec![IndicatorOutput { name: "value".to_string(), values }];
    }
    let mut cum_pv = 0.0;
    let mut cum_vol = 0.0;
    for i in anchor..len {
        let tp = (bars[i].high + bars[i].low + bars[i].close) / 3.0;
        cum_pv += tp * bars[i].volume;
        cum_vol += bars[i].volume;
        if cum_vol > 0.0 {
            values[i] = cum_pv / cum_vol;
        }
    }
    vec![IndicatorOutput { name: "value".to_string(), values }]
}

pub fn anchored_vwap_store(
    store: &CandleStore,
    anchor: usize,
    _nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let len = store.len();
    let mut values = vec![f64::NAN; len];
    if anchor >= len {
        return vec![IndicatorOutput { name: "value".to_string(), values }];
    }
    let mut cum_pv = 0.0;
    let mut cum_vol = 0.0;
    for i in anchor..len {
        let tp = (store.high[i] + store.low[i] + store.close[i]) / 3.0;
        cum_pv += tp * store.volume[i];
        cum_vol += store.volume[i];
        if cum_vol > 0.0 {
            values[i] = cum_pv / cum_vol;
        }
    }
    vec![IndicatorOutput { name: "value".to_string(), values }]
}

/// For incremental update, recompute from anchor to current bar.
/// We store cumulative_pv and cumulative_volume as hidden state.
pub fn latest_anchored_vwap_store(
    store: &CandleStore,
    anchor: usize,
    outputs: &crate::types::IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let len = store.len();
    if len == 0 || anchor >= len {
        return (None, None, None);
    }
    let i = len - 1;
    let tp = (store.high[i] + store.low[i] + store.close[i]) / 3.0;

    let prev_pv = outputs
        .get("cumulative_pv")
        .and_then(|s| s.get(i.saturating_sub(1)).copied())
        .and_then(nan_to_none)
        .unwrap_or(0.0);
    let prev_vol = outputs
        .get("cumulative_volume")
        .and_then(|s| s.get(i.saturating_sub(1)).copied())
        .and_then(nan_to_none)
        .unwrap_or(0.0);

    // If we're before the anchor, zero
    if i < anchor {
        return (None, Some(0.0), Some(0.0));
    }

    let cum_pv = prev_pv + tp * store.volume[i];
    let cum_vol = prev_vol + store.volume[i];
    let value = if cum_vol > 0.0 {
        Some(cum_pv / cum_vol)
    } else {
        None
    };
    (value, Some(cum_pv), Some(cum_vol))
}

use crate::NodeCache;
use crate::{Bar, CandleStore, IndicatorOutput, Series};
use std::collections::HashMap;

/// Darvas Box Theory:
/// - A new box top is established when a new high is made and confirmed
///   (3 consecutive bars not exceeding it).
/// - The box bottom is the lowest low during the confirmation period.
/// - The box remains until price breaks above the top (new box starts)
///   or breaks below the bottom (exit signal).
///
/// Outputs: top (upper box boundary), bottom (lower box boundary)
pub fn darvas_box(bars: &[Bar], _nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let len = bars.len();
    let mut top = vec![f64::NAN; len];
    let mut bottom = vec![f64::NAN; len];
    if len < 4 {
        return vec![
            IndicatorOutput { name: "top".to_string(), values: top },
            IndicatorOutput { name: "bottom".to_string(), values: bottom },
        ];
    }

    let mut box_top = f64::NAN;
    let mut box_bottom = f64::NAN;
    let mut high_candidate = bars[0].high;
    let mut confirm_count = 0u32;
    let mut lowest_during_confirm = bars[0].low;
    let mut box_active = false;

    for i in 1..len {
        if box_active {
            // Check for breakout
            if bars[i].high > box_top {
                // Breakout above — start new box candidate
                box_active = false;
                high_candidate = bars[i].high;
                confirm_count = 0;
                lowest_during_confirm = bars[i].low;
            }
            // Box remains
            top[i] = box_top;
            bottom[i] = box_bottom;
        } else {
            // Looking for new box top confirmation
            if bars[i].high > high_candidate {
                // New high — reset confirmation
                high_candidate = bars[i].high;
                confirm_count = 0;
                lowest_during_confirm = bars[i].low;
            } else {
                // Bar did not exceed high_candidate
                confirm_count += 1;
                lowest_during_confirm = lowest_during_confirm.min(bars[i].low);

                if confirm_count >= 3 {
                    // Box confirmed
                    box_top = high_candidate;
                    box_bottom = lowest_during_confirm;
                    box_active = true;
                    top[i] = box_top;
                    bottom[i] = box_bottom;
                }
            }
        }
    }

    vec![
        IndicatorOutput { name: "top".to_string(), values: top },
        IndicatorOutput { name: "bottom".to_string(), values: bottom },
    ]
}

pub fn darvas_box_store(store: &CandleStore, _nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let len = store.len();
    let mut top = vec![f64::NAN; len];
    let mut bottom = vec![f64::NAN; len];
    if len < 4 {
        return vec![
            IndicatorOutput { name: "top".to_string(), values: top },
            IndicatorOutput { name: "bottom".to_string(), values: bottom },
        ];
    }

    let mut box_top = f64::NAN;
    let mut box_bottom = f64::NAN;
    let mut high_candidate = store.high[0];
    let mut confirm_count = 0u32;
    let mut lowest_during_confirm = store.low[0];
    let mut box_active = false;

    for i in 1..len {
        if box_active {
            if store.high[i] > box_top {
                box_active = false;
                high_candidate = store.high[i];
                confirm_count = 0;
                lowest_during_confirm = store.low[i];
            }
            top[i] = box_top;
            bottom[i] = box_bottom;
        } else {
            if store.high[i] > high_candidate {
                high_candidate = store.high[i];
                confirm_count = 0;
                lowest_during_confirm = store.low[i];
            } else {
                confirm_count += 1;
                lowest_during_confirm = lowest_during_confirm.min(store.low[i]);
                if confirm_count >= 3 {
                    box_top = high_candidate;
                    box_bottom = lowest_during_confirm;
                    box_active = true;
                    top[i] = box_top;
                    bottom[i] = box_bottom;
                }
            }
        }
    }

    vec![
        IndicatorOutput { name: "top".to_string(), values: top },
        IndicatorOutput { name: "bottom".to_string(), values: bottom },
    ]
}

pub fn latest_darvas_box_store(store: &CandleStore) -> (Option<f64>, Option<f64>) {
    let outputs = darvas_box_store(store, &mut HashMap::new());
    let t = outputs[0].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    let b = outputs[1].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    (t, b)
}

use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// ZigZag: connects significant highs and lows based on a percentage threshold.
/// The multiplier parameter is used as the percentage threshold (default 5%).
/// Only changes direction when price moves by at least threshold% from the last pivot.
/// Intermediate values are linearly interpolated.
pub fn zigzag_store(store: &CandleStore, threshold_pct: f64, _nodes: &mut NodeCache) -> RcSeries {
    let key = format!("zigzag:hl:{}", threshold_pct);
    if let Some(v) = _nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 || threshold_pct <= 0.0 {
        let rc = Rc::new(out);
        _nodes.insert(key, Rc::clone(&rc));
        return rc;
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
            if store.high[i] > last_high {
                last_high = store.high[i];
                last_high_idx = i;
            }
            if store.low[i] < last_high * (1.0 - threshold) {
                // Reversal down
                pivots.push((last_high_idx, last_high));
                trend = -1;
                last_low = store.low[i];
                last_low_idx = i;
            }
        }
        if trend <= 0 {
            if store.low[i] < last_low {
                last_low = store.low[i];
                last_low_idx = i;
            }
            if store.high[i] > last_low * (1.0 + threshold) {
                // Reversal up
                pivots.push((last_low_idx, last_low));
                trend = 1;
                last_high = store.high[i];
                last_high_idx = i;
            }
        }
        if trend == 0 {
            if store.high[i] > last_high {
                last_high = store.high[i];
                last_high_idx = i;
            }
            if store.low[i] < last_low {
                last_low = store.low[i];
                last_low_idx = i;
            }
            if last_high > store.close[0] * (1.0 + threshold) {
                trend = 1;
            } else if last_low < store.close[0] * (1.0 - threshold) {
                trend = -1;
            }
        }
    }
    // Add final point
    if trend == 1 {
        pivots.push((last_high_idx, last_high));
    } else if trend == -1 {
        pivots.push((last_low_idx, last_low));
    } else {
        pivots.push((len - 1, store.close[len - 1]));
    }
    // Interpolate between pivots
    for w in pivots.windows(2) {
        let (i0, v0) = w[0];
        let (i1, v1) = w[1];
        if i1 == i0 {
            out[i0] = v0;
            continue;
        }
        for j in i0..=i1 {
            let t = (j - i0) as f64 / (i1 - i0) as f64;
            out[j] = v0 + t * (v1 - v0);
        }
    }
    let rc = Rc::new(out);
    _nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_zigzag_store(store: &CandleStore, threshold_pct: f64) -> Option<f64> {
    zigzag_store(store, threshold_pct, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "ZIGZAG",
        name: "ZIGZAG",
        category: "Trend Analysis",
        pane: "overlay",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "multiplier",
            label: "Threshold %",
            default: 5.0,
            min: 0.1,
            step: "0.1",
        }],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "overlay", "#2563eb",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            vec![1.0; len],
        )
    }

    fn assert_series_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else {
                assert!((actual - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn zigzag_stays_flat_for_constant_prices() {
        let store = ohlc_store(&[10.0, 10.0, 10.0, 10.0]);
        let values = zigzag_store(&store, 5.0, &mut HashMap::new());

        assert_series_close(&values, &[10.0, 10.0, 10.0, 10.0]);
        assert_eq!(latest_zigzag_store(&store, 5.0), Some(10.0));
    }

    #[test]
    fn zigzag_marks_reversals_once_the_threshold_is_crossed() {
        let store = ohlc_store(&[10.0, 12.0, 9.0, 13.0]);
        let values = zigzag_store(&store, 10.0, &mut HashMap::new());

        assert_series_close(&values, &[10.0, 12.0, 9.0, 13.0]);
        assert_eq!(latest_zigzag_store(&store, 10.0), Some(13.0));
    }
}

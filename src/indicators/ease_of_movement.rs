use crate::indicators::derived::hl2_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Ease of Movement (EMV):
/// Distance Moved = ((high + low)/2 - (prev_high + prev_low)/2)
/// Box Ratio = volume / (high - low)
/// EMV = Distance Moved / Box Ratio (then smoothed with SMA of period)
pub fn ease_of_movement_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("emv:hlv:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let hl2 = hl2_store(store, nodes);
    let mut raw = vec![f64::NAN; len];
    for i in 1..len {
        let distance = hl2[i] - hl2[i - 1];
        let hl_diff = store.high[i] - store.low[i];
        if hl_diff.abs() > 1e-10 && store.volume[i] > 0.0 {
            let box_ratio = (store.volume[i] / 10000.0) / hl_diff;
            if box_ratio.abs() > 1e-10 {
                raw[i] = distance / box_ratio;
            }
        }
    }
    // SMA smoothing
    let mut out = vec![f64::NAN; len];
    if period > 0 && len >= period {
        for i in period..len {
            let window = &raw[i + 1 - period..=i];
            let valid: Vec<f64> = window.iter().filter(|v| !v.is_nan()).copied().collect();
            if !valid.is_empty() {
                out[i] = valid.iter().sum::<f64>() / valid.len() as f64;
            }
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_ease_of_movement_store(store: &CandleStore, period: usize) -> Option<f64> {
    ease_of_movement_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "EASE_OF_MOVEMENT",
        "EASE OF MOVEMENT",
        "Momentum/Oscillator",
        "separate",
        14,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(_, _, volume)| *volume).collect(),
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
    fn ease_of_movement_is_smoothed_distance_over_box_ratio() {
        let store = ohlcv_store(&[
            (2.0, 0.0, 10000.0),
            (3.0, 1.0, 10000.0),
            (4.0, 2.0, 10000.0),
        ]);
        let values = ease_of_movement_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 2.0]);
        assert_eq!(latest_ease_of_movement_store(&store, 2), Some(2.0));
    }
}

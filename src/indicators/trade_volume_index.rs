use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Trade Volume Index (TVI):
/// Cumulative volume based on tick direction.
/// If close > prev_close: add volume
/// If close < prev_close: subtract volume
/// If close == prev_close: no change
pub fn trade_volume_index_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "tvi:cv".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len == 0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    out[0] = 0.0;
    for i in 1..len {
        let diff = store.close[i] - store.close[i - 1];
        if diff > 0.0 {
            out[i] = out[i - 1] + store.volume[i];
        } else if diff < 0.0 {
            out[i] = out[i - 1] - store.volume[i];
        } else {
            out[i] = out[i - 1];
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_trade_volume_index_store(store: &CandleStore, prev: Option<&[f64]>) -> Option<f64> {
    let len = store.len();
    if len == 0 {
        return None;
    }
    if len == 1 {
        return Some(0.0);
    }
    let prev_tvi = prev
        .and_then(|s| s.get(len - 2).copied())
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
        .unwrap_or(0.0);
    let diff = store.close[len - 1] - store.close[len - 2];
    if diff > 0.0 {
        Some(prev_tvi + store.volume[len - 1])
    } else if diff < 0.0 {
        Some(prev_tvi - store.volume[len - 1])
    } else {
        Some(prev_tvi)
    }
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "TRADE_VOLUME_INDEX",
        name: "TRADE VOLUME INDEX",
        category: "Money Flow",
        pane: "separate",
        params: Vec::new(),
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#0f766e",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(_, volume)| *volume).collect(),
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
    fn trade_volume_index_accumulates_volume_by_direction() {
        let store = ohlcv_store(&[(1.0, 10.0), (2.0, 20.0), (1.0, 30.0), (1.0, 40.0)]);
        let values = trade_volume_index_store(&store, &mut HashMap::new());

        assert_series_close(&values, &[0.0, 20.0, -10.0, -10.0]);
        assert_eq!(
            latest_trade_volume_index_store(&store, Some(&values[..])),
            Some(-10.0)
        );
    }
}

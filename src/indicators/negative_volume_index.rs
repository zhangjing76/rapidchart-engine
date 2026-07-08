use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Negative Volume Index (NVI):
/// Starts at 1000. Only changes on days when volume decreases.
/// NVI = prev_NVI + (close - prev_close) / prev_close * prev_NVI  (when volume < prev_volume)
pub fn negative_volume_index_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "nvi:cv".to_string();
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
    out[0] = 1000.0;
    for i in 1..len {
        if store.volume[i] < store.volume[i - 1] && store.close[i - 1] != 0.0 {
            let roc = (store.close[i] - store.close[i - 1]) / store.close[i - 1];
            out[i] = out[i - 1] + roc * out[i - 1];
        } else {
            out[i] = out[i - 1];
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_negative_volume_index_store(
    store: &CandleStore,
    prev: Option<&[f64]>,
) -> Option<f64> {
    let len = store.len();
    if len == 0 {
        return None;
    }
    if len == 1 {
        return Some(1000.0);
    }
    let prev_nvi = prev
        .and_then(|s| s.get(len - 2).copied())
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
        .unwrap_or(1000.0);
    if store.volume[len - 1] < store.volume[len - 2] && store.close[len - 2] != 0.0 {
        let roc = (store.close[len - 1] - store.close[len - 2]) / store.close[len - 2];
        Some(prev_nvi + roc * prev_nvi)
    } else {
        Some(prev_nvi)
    }
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "NEGATIVE_VOLUME_INDEX",
        name: "NEGATIVE VOLUME INDEX",
        category: "Trend Analysis",
        pane: "separate",
        params: Vec::new(),
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#dc2626",
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
    fn nvi_only_changes_on_volume_decreases() {
        let store = ohlcv_store(&[(10.0, 3.0), (11.0, 2.0), (12.0, 3.0)]);
        let values = negative_volume_index_store(&store, &mut HashMap::new());

        assert_series_close(&values, &[1000.0, 1100.0, 1100.0]);
        assert_eq!(
            latest_negative_volume_index_store(&store, Some(&values[..])),
            Some(1100.0)
        );
    }
}

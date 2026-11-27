use crate::NodeCache;
use crate::{CandleStore, NamedSeries, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// The index's starting level.
const BASE: f64 = 100.0;

/// Negative Volume Index (NVI):
/// Starts at 100. Only changes on days when volume decreases.
/// NVI = prev_NVI + (close - prev_close) / prev_close * prev_NVI  (when volume < prev_volume)
pub fn negative_volume_index_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "nvi100:cv".to_string();
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
    out[0] = BASE;
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
        return Some(BASE);
    }
    let prev_nvi = prev
        .and_then(|s| s.get(len - 2).copied())
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
        .unwrap_or(BASE);
    if store.volume[len - 1] < store.volume[len - 2] && store.close[len - 2] != 0.0 {
        let roc = (store.close[len - 1] - store.close[len - 2]) / store.close[len - 2];
        Some(prev_nvi + roc * prev_nvi)
    } else {
        Some(prev_nvi)
    }
}

/// The index with its moving-average line: SMA(period) of the index.
pub fn negative_volume_index_outputs(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<NamedSeries> {
    let index = negative_volume_index_store(store, nodes);
    let ma = crate::indicators::kst::sma_from_series(&index, period);
    vec![
        crate::named_series("value", index),
        crate::named_series("ma", ma),
    ]
}

pub fn latest_negative_volume_index_ma(store: &CandleStore, period: usize) -> Option<f64> {
    let index = negative_volume_index_store(store, &mut HashMap::new());
    crate::indicators::kst::last_sma(&index, period)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "NEGATIVE_VOLUME_INDEX",
        name: "NEGATIVE VOLUME INDEX",
        category: "Trend Analysis",
        pane: "separate",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "period",
            label: "MA Period",
            default: 255.0,
            min: 1.0,
            step: "1",
        }],
        outputs: vec![
            crate::descriptors::output_descriptor("value", "line", "separate", "#dc2626"),
            crate::descriptors::output_descriptor("ma", "line", "separate", "#64748b"),
        ],
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

        assert_series_close(&values, &[100.0, 110.0, 110.0]);
        assert_eq!(
            latest_negative_volume_index_store(&store, Some(&values[..])),
            Some(110.0)
        );
    }
}

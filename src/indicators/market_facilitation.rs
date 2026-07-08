use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Market Facilitation Index: (high - low) / volume
/// Measures price movement efficiency relative to volume.
pub fn market_facilitation_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "mfi_bw:hlv".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        if store.volume[i] > 0.0 {
            out[i] = (store.high[i] - store.low[i]) / store.volume[i];
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_market_facilitation_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 {
        return None;
    }
    let i = store.len() - 1;
    if store.volume[i] > 0.0 {
        Some((store.high[i] - store.low[i]) / store.volume[i])
    } else {
        None
    }
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "MARKET_FACILITATION",
        name: "MARKET FACILITATION INDEX",
        category: "Volume",
        pane: "separate",
        params: Vec::new(),
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#9333ea",
        )],
    }
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
    fn market_facilitation_is_range_divided_by_volume() {
        let store = ohlcv_store(&[(10.0, 6.0, 2.0), (14.0, 8.0, 4.0)]);
        let values = market_facilitation_store(&store, &mut HashMap::new());

        assert_series_close(&values, &[2.0, 1.5]);
        assert_eq!(latest_market_facilitation_store(&store), Some(1.5));
    }
}

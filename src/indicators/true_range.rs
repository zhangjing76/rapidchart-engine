use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// True Range: max(H-L, |H-PC|, |L-PC|) per bar. First bar = H-L.
pub fn true_range_series_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "true_range:hlc".to_string();
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len == 0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    out[0] = store.high[0] - store.low[0];
    for i in 1..len {
        out[i] = (store.high[i] - store.low[i])
            .max((store.high[i] - store.close[i - 1]).abs())
            .max((store.low[i] - store.close[i - 1]).abs());
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_true_range_store(store: &CandleStore) -> Option<f64> {
    let len = store.len();
    if len == 0 {
        return None;
    }
    if len == 1 {
        return Some(store.high[0] - store.low[0]);
    }
    let i = len - 1;
    Some(
        (store.high[i] - store.low[i])
            .max((store.high[i] - store.close[i - 1]).abs())
            .max((store.low[i] - store.close[i - 1]).abs()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
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
    fn true_range_is_the_manual_bar_range() {
        let store = ohlc_store(&[
            (10.0, 8.0, 9.0),
            (12.0, 7.0, 11.0),
            (13.0, 9.0, 10.0),
        ]);
        let values = true_range_series_store(&store, &mut HashMap::new());

        assert_series_close(&values, &[2.0, 5.0, 4.0]);
        assert_eq!(latest_true_range_store(&store), Some(4.0));
    }
}

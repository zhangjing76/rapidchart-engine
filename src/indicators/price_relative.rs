use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Price Relative: ratio of current close to close N bars ago.
/// value[i] = close[i] / close[i - period]
/// This shows relative strength over the lookback period.
pub fn price_relative_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("price_relative:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len <= period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period..len {
        let prev = store.close[i - period];
        if prev != 0.0 {
            out[i] = store.close[i] / prev;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_price_relative_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    let i = store.len() - 1;
    let prev = store.close[i - period];
    if prev == 0.0 {
        None
    } else {
        Some(store.close[i] / prev)
    }
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
                "PRICE_RELATIVE",
                "PRICE RELATIVE",
                "Compare",
                "separate",
                14,
            )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn close_store(values: &[f64]) -> CandleStore {
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
    fn price_relative_is_the_manual_ratio() {
        let store = close_store(&[1.0, 2.0, 4.0]);
        let values = price_relative_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 4.0]);
        assert_eq!(latest_price_relative_store(&store, 2), Some(4.0));
    }

    #[test]
    fn price_relative_uses_the_selected_lookback() {
        let store = close_store(&[2.0, 3.0, 9.0]);
        let values = price_relative_store(&store, 1, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, 1.5, 3.0]);
        assert_eq!(latest_price_relative_store(&store, 1), Some(3.0));
    }
}

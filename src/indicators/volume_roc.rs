use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Volume Rate of Change: (volume - volume[period]) / volume[period] * 100
pub fn volume_roc_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("vol_roc:v:{period}");
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len <= period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period..len {
        let prev = store.volume[i - period];
        if prev > 0.0 {
            out[i] = ((store.volume[i] - prev) / prev) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_volume_roc_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    let i = store.len() - 1;
    let prev = store.volume[i - period];
    if prev > 0.0 {
        Some(((store.volume[i] - prev) / prev) * 100.0)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
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
    fn volume_roc_is_the_manual_percentage_change() {
        let store = ohlcv_store(&[2.0, 4.0, 8.0]);
        let values = volume_roc_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 300.0]);
        assert_eq!(latest_volume_roc_store(&store, 2), Some(300.0));
    }
}

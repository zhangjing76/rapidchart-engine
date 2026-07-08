use crate::indicators::sma::sma_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// RAVI (Range Action Verification Index):
/// |SMA(short) - SMA(long)| / SMA(long) * 100
pub fn ravi_store(
    store: &CandleStore,
    short: usize,
    long: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("ravi:close:{}:{}", short, long);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma_short = sma_close_store(store, short, nodes);
    let sma_long = sma_close_store(store, long, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let s = sma_short[i];
        let l = sma_long[i];
        if !s.is_nan() && !l.is_nan() && l.abs() > 1e-10 {
            out[i] = ((s - l).abs() / l) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_ravi_store(store: &CandleStore, short: usize, long: usize) -> Option<f64> {
    ravi_store(store, short, long, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
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
    fn ravi_is_the_absolute_sma_divergence() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0]);
        let values = ravi_store(&store, 2, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 25.0, 16.666666666666664]);
        assert_eq!(latest_ravi_store(&store, 2, 3), Some(16.666666666666664));
    }
}

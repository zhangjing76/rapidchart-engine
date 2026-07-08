use crate::indicators::bollinger::bollinger_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Bollinger %b: (close - lower) / (upper - lower)
/// Values above 1 = above upper band, below 0 = below lower band.
const UPPER_SLOT: usize = 0;
const LOWER_SLOT: usize = 2;

pub fn bollinger_pct_b_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("bb_pctb:{}:{}", period, multiplier);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let bb = bollinger_store(store, period, multiplier, nodes);
    let upper = &bb[UPPER_SLOT].values;
    let lower = &bb[LOWER_SLOT].values;
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let u = upper[i];
        let l = lower[i];
        if !u.is_nan() && !l.is_nan() && (u - l).abs() > 1e-10 {
            out[i] = (store.close[i] - l) / (u - l);
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_bollinger_pct_b_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> Option<f64> {
    bollinger_pct_b_store(store, period, multiplier, &mut HashMap::new())
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
    fn bollinger_pct_b_is_half_on_the_middle_band() {
        let store = close_store(&[1.0, 2.0, 3.0]);
        let values = bollinger_pct_b_store(&store, 3, 2.0, &mut HashMap::new());
        let band = (2.0_f64 / 3.0).sqrt() * 2.0;
        let expected = (3.0 - (2.0 - band)) / (2.0 * band);

        assert_series_close(&values, &[f64::NAN, f64::NAN, expected]);
        assert_eq!(latest_bollinger_pct_b_store(&store, 3, 2.0), Some(expected));
    }
}

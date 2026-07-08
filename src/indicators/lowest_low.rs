use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Lowest Low Value over a rolling window of `period` bars.

pub fn lowest_low_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("lowest_low:l:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period > 0 && store.len() >= period {
        for i in period - 1..store.len() {
            let min = store.low[i + 1 - period..=i]
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b));
            out[i] = min;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_lowest_low_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let min = store.low[start..]
        .iter()
        .fold(f64::INFINITY, |a, &b| a.min(b));
    Some(min)
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
    fn lowest_low_is_the_rolling_minimum() {
        let store = ohlc_store(&[
            (10.0, 6.0, 9.0),
            (12.0, 7.0, 10.0),
            (11.0, 5.0, 10.0),
            (14.0, 9.0, 13.0),
        ]);
        let values = lowest_low_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 5.0, 5.0]);
        assert_eq!(latest_lowest_low_store(&store, 3), Some(5.0));
    }
}

use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Highest High Value over a rolling window of `period` bars.

pub fn highest_high_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("highest_high:h:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period > 0 && store.len() >= period {
        for i in period - 1..store.len() {
            let max = store.high[i + 1 - period..=i]
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            out[i] = max;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_highest_high_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let max = store.high[start..]
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    Some(max)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
                "HIGHEST_HIGH",
                "HIGHEST HIGH VALUE",
                "Statistical",
                "overlay",
                14,
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
    fn highest_high_is_the_rolling_maximum() {
        let store = ohlc_store(&[
            (10.0, 6.0, 9.0),
            (12.0, 7.0, 10.0),
            (11.0, 8.0, 10.0),
            (14.0, 9.0, 13.0),
        ]);
        let values = highest_high_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 12.0, 14.0]);
        assert_eq!(latest_highest_high_store(&store, 3), Some(14.0));
    }
}

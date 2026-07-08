use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn williams_r_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("willr:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
        let window = index + 1 - period..=index;
        let highest_high = window
            .clone()
            .map(|i| store.high[i])
            .fold(f64::MIN, f64::max);
        let lowest_low = window.map(|i| store.low[i]).fold(f64::MAX, f64::min);
        let range = highest_high - lowest_low;
        *item = if range == 0.0 {
            0.0
        } else {
            -100.0 * (highest_high - store.close[index]) / range
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_williams_r_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let highest_high = store.high[start..].iter().copied().fold(f64::MIN, f64::max);
    let lowest_low = store.low[start..].iter().copied().fold(f64::MAX, f64::min);
    let range = highest_high - lowest_low;
    Some(if range == 0.0 {
        0.0
    } else {
        -100.0 * (highest_high - store.close[store.len() - 1]) / range
    })
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
    fn williams_r_is_the_manual_position_in_range() {
        let store = ohlc_store(&[(4.0, 0.0, 3.0), (6.0, 2.0, 5.0), (6.0, 0.0, 5.0)]);
        let values = williams_r_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, -16.666666666666664, -16.666666666666664]);
        assert_eq!(
            latest_williams_r_store(&store, 2),
            Some(-16.666666666666668)
        );
    }
}

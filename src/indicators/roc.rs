use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn roc_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("roc:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period) {
        let previous = store.close[index - period];
        *item = if previous == 0.0 {
            0.0
        } else {
            100.0 * (store.close[index] / previous - 1.0)
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_roc_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    let previous = store.close[store.len() - 1 - period];
    Some(if previous == 0.0 {
        0.0
    } else {
        100.0 * (store.close[store.len() - 1] / previous - 1.0)
    })
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
    fn roc_is_the_manual_rate_of_change() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0]);
        let values = roc_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 200.0, 100.0]);
        assert_eq!(latest_roc_store(&store, 2), Some(100.0));
    }
}

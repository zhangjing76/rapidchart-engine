use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Chande Momentum Oscillator (CMO):
/// CMO = ((sum_up - sum_down) / (sum_up + sum_down)) * 100
/// where sum_up = sum of positive changes over period,
///       sum_down = sum of absolute negative changes over period.
pub fn chande_momentum_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("cmo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period + 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period..len {
        let mut sum_up = 0.0;
        let mut sum_down = 0.0;
        for j in (i + 1 - period)..=i {
            let diff = store.close[j] - store.close[j - 1];
            if diff > 0.0 {
                sum_up += diff;
            } else {
                sum_down += -diff;
            }
        }
        let total = sum_up + sum_down;
        if total > 0.0 {
            out[i] = ((sum_up - sum_down) / total) * 100.0;
        } else {
            out[i] = 0.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_chande_momentum_store(store: &CandleStore, period: usize) -> Option<f64> {
    chande_momentum_store(store, period, &mut HashMap::new())
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
    fn cmo_is_the_manual_sum_of_signed_changes() {
        let store = close_store(&[1.0, 2.0, 3.0]);
        let values = chande_momentum_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 100.0]);
        assert_eq!(latest_chande_momentum_store(&store, 2), Some(100.0));
    }
}

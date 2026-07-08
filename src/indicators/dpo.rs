use crate::indicators::sma::{latest_sma_store, sma_close_store};
use crate::rc_into_owned;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn dpo_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("dpo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma_values = rc_into_owned(sma_close_store(store, period, nodes));
    let shift = period / 2 + 1;
    let mut out = vec![f64::NAN; store.len()];
    for (index, (out_val, &mean)) in out.iter_mut().zip(sma_values.iter()).enumerate() {
        if index < period.saturating_sub(1) || index < shift {
            continue;
        }
        if !mean.is_nan() {
            *out_val = store.close[index - shift] - mean;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_dpo_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let shift = period / 2 + 1;
    let index = store.len() - 1;
    if index < shift || index < period.saturating_sub(1) {
        return None;
    }
    latest_sma_store(store, period).map(|mean| store.close[index - shift] - mean)
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
    fn dpo_is_zero_for_flat_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0, 10.0, 10.0]);
        let values = dpo_store(&store, 4, &mut HashMap::new());

        assert_series_close(
            &values,
            &[f64::NAN, f64::NAN, f64::NAN, 0.0, 0.0, 0.0],
        );
        assert_eq!(latest_dpo_store(&store, 4), Some(0.0));
    }
}

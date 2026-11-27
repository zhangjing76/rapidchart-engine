use crate::indicators::sma::sma_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Disparity Index: ((close - SMA(close, period)) / SMA(close, period)) * 100
/// Measures percentage distance of close from its moving average.
pub fn disparity_index_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("disparity_sma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma = sma_close_store(store, period, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let e = sma[i];
        if !e.is_nan() && e.abs() > 1e-10 {
            out[i] = ((store.close[i] - e) / e) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_disparity_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    disparity_index_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "DISPARITY_INDEX",
        "DISPARITY INDEX",
        "Momentum/Oscillator",
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
    fn disparity_index_is_zero_when_price_matches_the_average() {
        let store = close_store(&[5.0, 5.0, 5.0]);
        let values = disparity_index_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, 0.0, 0.0]);
        assert_eq!(latest_disparity_index_store(&store, 2), Some(0.0));
    }

    #[test]
    fn disparity_index_measures_the_close_against_its_simple_average() {
        let store = close_store(&[1.0, 2.0, 3.0, 6.0]);
        let values = disparity_index_store(&store, 2, &mut HashMap::new());

        // Bar 3: the average of 3 and 6 is 4.5, and 6 is a third above it.
        assert_series_close(&values, &[f64::NAN, 100.0 / 3.0, 20.0, 100.0 / 3.0]);
    }
}

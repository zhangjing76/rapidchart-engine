use crate::indicators::kst::sma_from_series;
use crate::indicators::sma::sma_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn trima_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("trima:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let values = sma_from_series(&sma_close_store(store, period, nodes), period);
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_trima_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    // TRIMA = SMA of SMA(close, period), period
    // We need at least 2*period - 1 bars for the last SMA(SMA) value to be valid
    let needed = 2 * period - 1;
    if store.len() < needed {
        return None;
    }
    // Compute SMA values for the last `period` positions
    let p = period as f64;
    let start = store.len() - needed;
    let mut sma_values = Vec::with_capacity(period);
    for i in (start + period - 1)..store.len() {
        let window = &store.close[i + 1 - period..=i];
        sma_values.push(window.iter().sum::<f64>() / p);
    }
    if sma_values.len() < period {
        return None;
    }
    // SMA of the last `period` SMA values
    let last_period = &sma_values[sma_values.len() - period..];
    Some(last_period.iter().sum::<f64>() / p)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("TRIMA", "TRIMA", "Moving Average", "overlay", 20)
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
    fn trima_is_the_input_when_prices_are_constant() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0, 10.0]);
        let values = trima_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, f64::NAN, f64::NAN, 10.0]);
        assert_eq!(latest_trima_store(&store, 3), Some(10.0));
    }

    #[test]
    fn trima_is_the_sma_of_the_sma() {
        let store = close_store(&[10.0, 12.0, 14.0, 16.0, 18.0]);
        let values = trima_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, f64::NAN, f64::NAN, 14.0]);
        assert_eq!(latest_trima_store(&store, 3), Some(14.0));
    }
}

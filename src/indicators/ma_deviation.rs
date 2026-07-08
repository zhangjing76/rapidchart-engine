use crate::indicators::sma::sma_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Moving Average Deviation: ((close - SMA) / SMA) * 100
pub fn ma_deviation_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ma_dev:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma = sma_close_store(store, period, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let s = sma[i];
        if !s.is_nan() && s.abs() > 1e-10 {
            out[i] = ((store.close[i] - s) / s) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_ma_deviation_store(store: &CandleStore, period: usize) -> Option<f64> {
    ma_deviation_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
                "MA_DEVIATION",
                "MOVING AVERAGE DEVIATION",
                "Momentum/Oscillator",
                "separate",
                20,
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
    fn ma_deviation_is_zero_when_price_matches_sma() {
        let store = close_store(&[5.0, 5.0, 5.0]);
        let values = ma_deviation_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, 0.0, 0.0]);
        assert_eq!(latest_ma_deviation_store(&store, 2), Some(0.0));
    }
}

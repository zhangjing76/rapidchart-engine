use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn sma_close_values(values: &[f64], period: usize) -> Series {
    let mut out = Vec::with_capacity(values.len());
    let mut sum = 0.0;
    for (index, value) in values.iter().copied().enumerate() {
        sum += value;
        if index >= period {
            sum -= values[index - period];
        }
        out.push(if period > 0 && index + 1 >= period {
            sum / period as f64
        } else {
            f64::NAN
        });
    }
    out
}
pub fn latest_sma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    Some(store.close[store.len() - period..].iter().sum::<f64>() / period as f64)
}
pub fn sma_close_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("sma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(sma_close_values(&store.close, period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("SMA", "SMA", "Moving Average", "overlay", 20)
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
    fn sma_is_the_manual_rolling_average() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let values = sma_close_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 2.0, 3.0, 4.0]);
        assert_eq!(latest_sma_store(&store, 3), Some(4.0));
    }
}

use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn stddev_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("stddev:close:{period}");
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
        let window = &store.close[index + 1 - period..=index];
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window
            .iter()
            .map(|close| {
                let diff = close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        *item = variance.sqrt();
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_stddev_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let window = &store.close[store.len() - period..];
    let mean = window.iter().sum::<f64>() / period as f64;
    let variance = window
        .iter()
        .map(|v| {
            let d = v - mean;
            d * d
        })
        .sum::<f64>()
        / period as f64;
    Some(variance.sqrt())
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("STDDEV", "STDDEV", "Statistical", "separate", 20)
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
    fn stddev_is_the_population_standard_deviation() {
        let store = close_store(&[1.0, 2.0, 3.0]);
        let values = stddev_store(&store, 3, &mut HashMap::new());
        let expected = (2.0_f64 / 3.0).sqrt();

        assert_series_close(&values, &[f64::NAN, f64::NAN, expected]);
        assert_eq!(latest_stddev_store(&store, 3), Some(expected));
    }
}

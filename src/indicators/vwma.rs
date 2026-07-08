use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn vwma_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("vwma:close:volume:{period}");
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
        let start = index + 1 - period;
        let volume_sum = store.volume[start..=index].iter().sum::<f64>();
        if volume_sum == 0.0 {
            continue;
        }
        let weighted_sum = (start..=index)
            .map(|window_index| store.close[window_index] * store.volume[window_index])
            .sum::<f64>();
        *item = weighted_sum / volume_sum;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_vwma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let volume_sum = store.volume[start..].iter().sum::<f64>();
    if volume_sum == 0.0 {
        return None;
    }
    let weighted_sum = (start..store.len())
        .map(|index| store.close[index] * store.volume[index])
        .sum::<f64>();
    Some(weighted_sum / volume_sum)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("VWMA", "VWMA", "Moving Average", "overlay", 20)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(_, volume)| *volume).collect(),
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
    fn vwma_is_the_volume_weighted_average() {
        let store = ohlcv_store(&[(1.0, 1.0), (3.0, 2.0), (5.0, 3.0)]);
        let values = vwma_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, 2.3333333333333335, 4.2]);
        assert_eq!(latest_vwma_store(&store, 2), Some(4.2));
    }
}

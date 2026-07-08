use crate::indicators::adl::money_flow_multiplier_parts;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn cmf_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("cmf:hlcv:{period}");
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
        let mut mfv_sum = 0.0;
        let mut volume_sum = 0.0;
        for window_index in start..=index {
            mfv_sum += money_flow_multiplier_parts(
                store.high[window_index],
                store.low[window_index],
                store.close[window_index],
            ) * store.volume[window_index];
            volume_sum += store.volume[window_index];
        }
        *item = if volume_sum != 0.0 {
            mfv_sum / volume_sum
        } else {
            f64::NAN
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_cmf_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let mut mfv_sum = 0.0;
    let mut volume_sum = 0.0;
    for index in start..store.len() {
        mfv_sum +=
            money_flow_multiplier_parts(store.high[index], store.low[index], store.close[index])
                * store.volume[index];
        volume_sum += store.volume[index];
    }
    (volume_sum != 0.0).then_some(mfv_sum / volume_sum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(high, _, _, _)| *high).collect(),
            values.iter().map(|(_, low, _, _)| *low).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(_, _, _, volume)| *volume).collect(),
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
    fn cmf_is_zero_when_close_is_midrange() {
        let store = ohlcv_store(&[
            (2.0, 0.0, 1.0, 10.0),
            (2.0, 0.0, 1.0, 20.0),
            (2.0, 0.0, 1.0, 30.0),
        ]);
        let values = cmf_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, 0.0, 0.0]);
        assert_eq!(latest_cmf_store(&store, 2), Some(0.0));
    }
}

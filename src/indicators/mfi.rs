use crate::indicators::cci::typical_price_at;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn mfi_value(positive_flow: f64, negative_flow: f64) -> f64 {
    if negative_flow == 0.0 {
        100.0
    } else {
        let money_ratio = positive_flow / negative_flow;
        100.0 - 100.0 / (1.0 + money_ratio)
    }
}
pub fn mfi_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("mfi:hlcv:{period}");
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
        let mut positive_flow = 0.0;
        let mut negative_flow = 0.0;
        for current in index + 1 - period..=index {
            let previous = current - 1;
            let previous_tp = typical_price_at(store, previous);
            let current_tp = typical_price_at(store, current);
            let raw_flow = current_tp * store.volume[current];
            if current_tp > previous_tp {
                positive_flow += raw_flow;
            } else if current_tp < previous_tp {
                negative_flow += raw_flow;
            }
        }
        *item = mfi_value(positive_flow, negative_flow);
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_mfi_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;
    for current in store.len() - period..store.len() {
        let previous = current - 1;
        let previous_tp = typical_price_at(store, previous);
        let current_tp = typical_price_at(store, current);
        let raw_flow = current_tp * store.volume[current];
        if current_tp > previous_tp {
            positive_flow += raw_flow;
        } else if current_tp < previous_tp {
            negative_flow += raw_flow;
        }
    }
    Some(mfi_value(positive_flow, negative_flow))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("MFI", "MFI", "Money Flow", "separate", 14)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[f64]) -> CandleStore {
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
    fn mfi_is_hundred_when_all_flow_is_positive() {
        let store = ohlcv_store(&[1.0, 2.0, 3.0]);
        let values = mfi_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 100.0]);
        assert_eq!(latest_mfi_store(&store, 2), Some(100.0));
    }
}

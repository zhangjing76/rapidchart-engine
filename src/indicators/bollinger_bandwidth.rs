use crate::indicators::bollinger::bollinger_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Bollinger Bandwidth: (upper - lower) / middle * 100
const UPPER_SLOT: usize = 0;
const MIDDLE_SLOT: usize = 1;
const LOWER_SLOT: usize = 2;

pub fn bollinger_bandwidth_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("bb_bw:{}:{}", period, multiplier);
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let bb = bollinger_store(store, period, multiplier, nodes);
    let upper = &bb[UPPER_SLOT].values;
    let middle = &bb[MIDDLE_SLOT].values;
    let lower = &bb[LOWER_SLOT].values;
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        if !upper[i].is_nan()
            && !middle[i].is_nan()
            && !lower[i].is_nan()
            && middle[i].abs() > 1e-10
        {
            out[i] = ((upper[i] - lower[i]) / middle[i]) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_bollinger_bandwidth_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> Option<f64> {
    bollinger_bandwidth_store(store, period, multiplier, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "BOLLINGER_BANDWIDTH",
                name: "BOLLINGER BANDWIDTH",
                category: "Volatility",
                pane: "separate",
                params: vec![
                    crate::descriptors::ParamDescriptor {
                        name: "period",
                        label: "Period",
                        default: 20.0,
                        min: 1.0,
                        step: "1",
                    },
                    crate::descriptors::ParamDescriptor {
                        name: "multiplier",
                        label: "Multiplier",
                        default: 2.0,
                        min: 0.1,
                        step: "0.1",
                    },
                ],
                outputs: vec![crate::descriptors::output_descriptor("value", "line", "separate", "#9333ea")],
            }
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
    fn bollinger_bandwidth_is_the_relative_band_span() {
        let store = close_store(&[1.0, 2.0, 3.0]);
        let values = bollinger_bandwidth_store(&store, 3, 2.0, &mut HashMap::new());
        let band = (2.0_f64 / 3.0).sqrt() * 2.0;
        let expected = (2.0 * band / 2.0) * 100.0;

        assert_series_close(&values, &[f64::NAN, f64::NAN, expected]);
        assert_eq!(
            latest_bollinger_bandwidth_store(&store, 3, 2.0),
            Some(expected)
        );
    }
}

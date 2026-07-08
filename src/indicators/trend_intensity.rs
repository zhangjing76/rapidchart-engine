use crate::indicators::sma::sma_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Trend Intensity Index:
/// Counts bars above SMA vs below SMA over period.
/// TII = (bars_above - bars_below) / period * 100
/// Range: -100 (all below) to +100 (all above)
pub fn trend_intensity_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("tii:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma = sma_close_store(store, period, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..len {
        let sma_val = sma[i];
        if sma_val.is_nan() {
            continue;
        }
        let mut above = 0i32;
        let mut below = 0i32;
        for j in i + 1 - period..=i {
            if store.close[j] > sma_val {
                above += 1;
            } else {
                below += 1;
            }
        }
        out[i] = ((above - below) as f64 / period as f64) * 100.0;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_trend_intensity_store(store: &CandleStore, period: usize) -> Option<f64> {
    trend_intensity_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "TREND_INTENSITY",
        "TREND INTENSITY INDEX",
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
    fn trend_intensity_is_negative_when_all_closes_equal_sma() {
        let store = close_store(&[5.0, 5.0, 5.0]);
        let values = trend_intensity_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, -100.0]);
        assert_eq!(latest_trend_intensity_store(&store, 3), Some(-100.0));
    }
}

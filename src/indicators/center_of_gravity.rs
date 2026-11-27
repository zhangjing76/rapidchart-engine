use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Center of Gravity oscillator (Ehlers):
/// COG = -Sum(close[t-j] * (j+1), j=0..period-1) / Sum(close[t-j], j=0..period-1)
/// The current bar has weight 1 and the oldest bar in the window weight period.
pub fn center_of_gravity_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("cog_ehlers:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..len {
        let window = &store.close[i + 1 - period..=i];
        let sum_weighted: f64 = window
            .iter()
            .rev()
            .enumerate()
            .map(|(bars_ago, &c)| c * (bars_ago + 1) as f64)
            .sum();
        let sum_close: f64 = window.iter().sum();
        if sum_close.abs() > 1e-10 {
            out[i] = -sum_weighted / sum_close;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_center_of_gravity_store(store: &CandleStore, period: usize) -> Option<f64> {
    center_of_gravity_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "CENTER_OF_GRAVITY",
        "CENTER OF GRAVITY",
        "Momentum/Oscillator",
        "separate",
        10,
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
    fn center_of_gravity_weights_the_current_bar_least() {
        let store = close_store(&[1.0, 2.0, 3.0]);
        let values = center_of_gravity_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, -10.0 / 6.0]);
        assert_eq!(latest_center_of_gravity_store(&store, 3), Some(-10.0 / 6.0));
    }
}

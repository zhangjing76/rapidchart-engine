use crate::indicators::ema::{ema_close_store, ema_series};
use crate::rc_into_owned;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn trix_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("trix:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema1 = rc_into_owned(ema_close_store(store, period, nodes));
    let ema2_key = format!("trix:ema2:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let ema3 = ema_series(&ema2, period);
    let mut out = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        {
            let previous = ema3[index - 1];
            let current = ema3[index];
            if !previous.is_nan() && !current.is_nan() {
                out[index] = if previous != 0.0 {
                    100.0 * (current / previous - 1.0)
                } else {
                    0.0
                };
            }
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_trix_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let last_close = match store.last_close() {
        Some(c) => c,
        None => return (None, None, None, None),
    };
    if store.len() == 1 {
        return (None, Some(last_close), Some(last_close), Some(last_close));
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let prev_ema1 = outputs
        .get("ema1")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(last_close);
    let ema1 = alpha * last_close + (1.0 - alpha) * prev_ema1;
    let prev_ema2 = outputs
        .get("ema2")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(ema1);
    let ema2 = alpha * ema1 + (1.0 - alpha) * prev_ema2;
    let prev_ema3 = outputs
        .get("ema3")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(ema2);
    let ema3 = alpha * ema2 + (1.0 - alpha) * prev_ema3;
    let value = if prev_ema3 != 0.0 {
        Some(100.0 * (ema3 / prev_ema3 - 1.0))
    } else {
        Some(0.0)
    };
    (value, Some(ema1), Some(ema2), Some(ema3))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("TRIX", "TRIX", "Momentum/Oscillator", "separate", 15)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::named_series;
    use crate::types::IndicatorArena;
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
    fn trix_is_zero_for_constant_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let values = trix_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn trix_is_the_rate_of_change_of_the_third_ema() {
        let store = close_store(&[10.0, 12.0, 14.0, 16.0]);
        let values = trix_store(&store, 3, &mut HashMap::new());

        assert_series_close(
            &values,
            &[
                f64::NAN,
                2.499999999999991,
                6.0975609756097615,
                9.195402298850585,
            ],
        );

        let arena = IndicatorArena::from_named_outputs(vec![
            named_series("ema1", vec![10.0, 11.0, 12.5, 14.25]),
            named_series("ema2", vec![10.0, 10.5, 11.5, 12.875]),
            named_series("ema3", vec![10.0, 10.25, 10.875, 11.875]),
        ]);
        assert_eq!(
            latest_trix_store(&store, 3, &arena),
            (
                Some(9.195402298850585),
                Some(14.25),
                Some(12.875),
                Some(11.875)
            )
        );
    }
}

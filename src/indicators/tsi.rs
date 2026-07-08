use crate::indicators::ema::ema_series;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn tsi_store(
    store: &CandleStore,
    long: usize,
    short: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("tsi:{long}:{short}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut momentum = vec![f64::NAN; store.len()];
    let mut abs_momentum = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        let value = store.close[index] - store.close[index - 1];
        momentum[index] = value;
        abs_momentum[index] = value.abs();
    }
    let ema1 = ema_series(&momentum, long);
    let ema2 = ema_series(&ema1, short);
    let abs_ema1 = ema_series(&abs_momentum, long);
    let abs_ema2 = ema_series(&abs_ema1, short);
    let values: Series = ema2
        .iter()
        .zip(abs_ema2.iter())
        .map(|(num, den)| match (num, den) {
            (num, den) if !num.is_nan() && !den.is_nan() && *den != 0.0 => 100.0 * *num / *den,
            (a, b) if !a.is_nan() && !b.is_nan() => 0.0,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_tsi_store(
    store: &CandleStore,
    long: usize,
    short: usize,
    outputs: &IndicatorArena,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    if store.len() < 2 {
        return (None, None, None, None, None);
    }
    let momentum = store.close[store.len() - 1] - store.close[store.len() - 2];
    let abs_momentum = momentum.abs();
    let alpha_long = 2.0 / (long as f64 + 1.0);
    let alpha_short = 2.0 / (short as f64 + 1.0);
    let prev_m_ema1 = outputs
        .get("m_ema1")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(momentum);
    let m_ema1 = alpha_long * momentum + (1.0 - alpha_long) * prev_m_ema1;
    let prev_m_ema2 = outputs
        .get("m_ema2")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(m_ema1);
    let m_ema2 = alpha_short * m_ema1 + (1.0 - alpha_short) * prev_m_ema2;
    let prev_a_ema1 = outputs
        .get("a_ema1")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(abs_momentum);
    let a_ema1 = alpha_long * abs_momentum + (1.0 - alpha_long) * prev_a_ema1;
    let prev_a_ema2 = outputs
        .get("a_ema2")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(a_ema1);
    let a_ema2 = alpha_short * a_ema1 + (1.0 - alpha_short) * prev_a_ema2;
    let value = if a_ema2 != 0.0 {
        Some(100.0 * m_ema2 / a_ema2)
    } else {
        Some(0.0)
    };
    (
        value,
        Some(m_ema1),
        Some(m_ema2),
        Some(a_ema1),
        Some(a_ema2),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn tsi_is_zero_for_flat_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let values = tsi_store(&store, 3, 2, &mut HashMap::new());
        assert!(values[0].is_nan());
        assert!((values[1] - 0.0).abs() < 1e-12);
        assert!((values[2] - 0.0).abs() < 1e-12);
        assert!((values[3] - 0.0).abs() < 1e-12);

        let arena = IndicatorArena::from_outputs(vec![]);
        assert_eq!(latest_tsi_store(&store, 3, 2, &arena).0, Some(0.0));
    }
}

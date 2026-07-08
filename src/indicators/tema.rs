use crate::indicators::ema::{ema_close_store, ema_series};
use crate::rc_into_owned;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn tema_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("tema:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema1 = rc_into_owned(ema_close_store(store, period, nodes));
    let ema2_key = format!("tema:ema2:{period}");
    let ema3_key = format!("tema:ema3:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let ema3 = nodes
        .get(&ema3_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema2, period));
    nodes.insert(ema3_key, Rc::new(ema3.clone()));
    let values: Vec<_> = ema1
        .iter()
        .zip(ema2.iter())
        .zip(ema3.iter())
        .map(|((first, second), third)| match (first, second, third) {
            (first, second, third) if !first.is_nan() && !second.is_nan() && !third.is_nan() => {
                3.0 * *first - 3.0 * *second + *third
            }
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_tema_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let last_close = match store.last_close() {
        Some(c) => c,
        None => return (None, None, None, None),
    };
    if store.len() == 1 {
        return (
            Some(last_close),
            Some(last_close),
            Some(last_close),
            Some(last_close),
        );
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
    (
        Some(3.0 * ema1 - 3.0 * ema2 + ema3),
        Some(ema1),
        Some(ema2),
        Some(ema3),
    )
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

    #[test]
    fn tema_is_the_input_when_prices_are_constant() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let values = tema_store(&store, 3, &mut HashMap::new());

        assert_eq!(&*values, &[10.0, 10.0, 10.0, 10.0]);
    }

    #[test]
    fn tema_matches_manual_triple_ema_combination() {
        let store = close_store(&[10.0, 12.0, 14.0, 16.0]);
        let values = tema_store(&store, 3, &mut HashMap::new());

        assert_eq!(&*values, &[10.0, 11.75, 13.875, 16.0]);

        let arena = IndicatorArena::from_named_outputs(vec![
            named_series("ema1", vec![10.0, 11.0, 12.5, 14.25]),
            named_series("ema2", vec![10.0, 10.5, 11.5, 12.875]),
            named_series("ema3", vec![10.0, 10.25, 10.875, 11.875]),
        ]);
        assert_eq!(
            latest_tema_store(&store, 3, &arena),
            (Some(16.0), Some(14.25), Some(12.875), Some(11.875))
        );
    }
}

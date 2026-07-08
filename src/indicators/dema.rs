use crate::indicators::ema::{ema_close_store, ema_series};
use crate::rc_into_owned;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn dema_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("dema:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema1 = rc_into_owned(ema_close_store(store, period, nodes));
    let ema2_key = format!("dema:ema2:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let values: Vec<_> = ema1
        .iter()
        .zip(ema2.iter())
        .map(|(first, second)| match (first, second) {
            (first, second) if !first.is_nan() && !second.is_nan() => 2.0 * *first - *second,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_dema_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let last_close = match store.last_close() {
        Some(c) => c,
        None => return (None, None, None),
    };
    if store.len() == 1 {
        return (Some(last_close), Some(last_close), Some(last_close));
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
    (Some(2.0 * ema1 - ema2), Some(ema1), Some(ema2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IndicatorArena;
    use crate::named_series;
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
    fn dema_is_the_input_when_prices_are_constant() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let outputs = dema_store(&store, 3, &mut HashMap::new());
        assert_eq!(&*outputs, &[10.0, 10.0, 10.0, 10.0]);

        let arena = IndicatorArena::from_named_outputs(vec![
            named_series("ema1", vec![10.0, 10.0, 10.0, 10.0]),
            named_series("ema2", vec![10.0, 10.0, 10.0, 10.0]),
        ]);
        assert_eq!(
            latest_dema_store(&store, 3, &arena),
            (Some(10.0), Some(10.0), Some(10.0))
        );
    }
}

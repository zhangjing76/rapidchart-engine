use crate::indicators::ema::ema_series;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn force_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("force:close:volume:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut raw = vec![f64::NAN; store.len()];
    for (index, item) in raw.iter_mut().enumerate().skip(1) {
        *item = (store.close[index] - store.close[index - 1]) * store.volume[index];
    }
    let values = ema_series(&raw, period);
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_force_index_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    if store.len() < 2 {
        return (None, None);
    }
    let raw = (store.close[store.len() - 1] - store.close[store.len() - 2])
        * store.volume[store.len() - 1];
    let alpha = 2.0 / (period as f64 + 1.0);
    let prev_ema = outputs
        .get("fi_ema")
        .and_then(|s| s.get(store.len() - 2).copied())
        .filter(|v| !v.is_nan())
        .unwrap_or(raw);
    let ema = alpha * raw + (1.0 - alpha) * prev_ema;
    (Some(ema), Some(ema))
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
    fn force_index_is_ema_of_price_change_times_volume() {
        let store = ohlcv_store(&[(1.0, 1.0), (2.0, 2.0), (4.0, 3.0)]);
        let values = force_index_store(&store, 2, &mut HashMap::new());
        let arena = crate::IndicatorArena::from_named_outputs(vec![crate::named_series(
            "fi_ema",
            values.clone(),
        )]);

        assert_series_close(&values, &[f64::NAN, 2.0, 4.666666666666667]);
        assert_eq!(
            latest_force_index_store(&store, 2, &arena),
            (Some(4.666666666666667), Some(4.666666666666667))
        );
    }
}

use crate::indicators::ema::{ema_next, ema_sma_seeded};
use crate::nan_to_none;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::rc::Rc;

const VALUE_SLOT: usize = 0;

/// Price change from the previous bar times volume; no value on the first bar.
fn raw_force(store: &CandleStore) -> Series {
    let mut raw = vec![f64::NAN; store.len()];
    for (index, item) in raw.iter_mut().enumerate().skip(1) {
        *item = (store.close[index] - store.close[index - 1]) * store.volume[index];
    }
    raw
}

/// Force Index: an EMA of price change x volume that starts with a simple average, so the first
/// value is at bar `period`.
pub fn force_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("force_sma:close:volume:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(ema_sma_seeded(&raw_force(store), period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

/// Latest Force Index. Steps on from the previous bar's value when there is one; otherwise
/// (while the average is starting) calculates it from the bars.
pub fn latest_force_index_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> Option<f64> {
    let len = store.len();
    if len < 2 {
        return None;
    }
    let raw = (store.close[len - 1] - store.close[len - 2]) * store.volume[len - 1];
    match outputs.value_at_slot(VALUE_SLOT, len - 2) {
        Some(previous) => Some(ema_next(raw, previous, period)),
        None => ema_sma_seeded(&raw_force(store), period)
            .last()
            .copied()
            .and_then(nan_to_none),
    }
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "FORCE_INDEX",
        "FORCE INDEX",
        "Money Flow",
        "separate",
        13,
    )
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
        // Raw force: none, 2 x 1 = 2, 3 x 2 = 6, 4 x 1 = 4.
        let store = ohlcv_store(&[(1.0, 1.0), (2.0, 2.0), (4.0, 3.0), (5.0, 4.0)]);
        let values = force_index_store(&store, 2, &mut HashMap::new());
        let nan = f64::NAN;

        // Period 2: starts at bar 2 with (2 + 6) / 2, then 4 x 2/3 + 4 x 1/3.
        assert_series_close(&values, &[nan, nan, 4.0, 4.0]);
    }

    #[test]
    fn latest_force_index_matches_the_full_calculation() {
        let bars: Vec<(f64, f64)> = (0..40)
            .map(|i| (50.0 + ((i * 7) % 11) as f64, 100.0 + (i % 5) as f64 * 10.0))
            .collect();
        for period in [1, 2, 13] {
            let full = force_index_store(&ohlcv_store(&bars), period, &mut HashMap::new());
            for len in 1..=bars.len() {
                let store = ohlcv_store(&bars[..len]);
                let previous =
                    crate::IndicatorArena::from_named_outputs(vec![crate::named_series(
                        "value",
                        full[..len - 1].to_vec(),
                    )]);
                let latest = latest_force_index_store(&store, period, &previous);
                let expect = Some(full[len - 1]).filter(|v| !v.is_nan());
                match (latest, expect) {
                    (Some(a), Some(b)) => {
                        assert!((a - b).abs() < 1e-9, "period {period} len {len}")
                    }
                    (a, b) => assert_eq!(a, b, "period {period} len {len}"),
                }
            }
        }
    }
}

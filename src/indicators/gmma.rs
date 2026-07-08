use crate::indicators::ema::{ema_close_store, latest_ema_store};
use crate::CandleStore;
use crate::NodeCache;

/// GMMA periods: short-term group and long-term group.
const SHORT_PERIODS: [usize; 6] = [3, 5, 8, 10, 12, 15];
const LONG_PERIODS: [usize; 6] = [30, 35, 40, 45, 50, 60];
const SHORT_3_SLOT: usize = 0;
const SHORT_5_SLOT: usize = 1;
const SHORT_8_SLOT: usize = 2;
const SHORT_10_SLOT: usize = 3;
const SHORT_12_SLOT: usize = 4;
const SHORT_15_SLOT: usize = 5;
const LONG_30_SLOT: usize = 6;
const LONG_35_SLOT: usize = 7;
const LONG_40_SLOT: usize = 8;
const LONG_45_SLOT: usize = 9;
const LONG_50_SLOT: usize = 10;
const LONG_60_SLOT: usize = 11;
const SHORT_SLOTS: [usize; 6] = [
    SHORT_3_SLOT,
    SHORT_5_SLOT,
    SHORT_8_SLOT,
    SHORT_10_SLOT,
    SHORT_12_SLOT,
    SHORT_15_SLOT,
];
const LONG_SLOTS: [usize; 6] = [
    LONG_30_SLOT,
    LONG_35_SLOT,
    LONG_40_SLOT,
    LONG_45_SLOT,
    LONG_50_SLOT,
    LONG_60_SLOT,
];

fn output_name(prefix: &str, period: usize) -> String {
    format!("{prefix}_{period}")
}

/// Guppy Multiple Moving Average: 12 EMA lines.

pub fn gmma_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    let mut outputs = Vec::with_capacity(12);
    for &p in &SHORT_PERIODS {
        outputs.push(crate::named_series(
            output_name("short", p),
            ema_close_store(store, p, nodes),
        ));
    }
    for &p in &LONG_PERIODS {
        outputs.push(crate::named_series(
            output_name("long", p),
            ema_close_store(store, p, nodes),
        ));
    }
    outputs
}

pub fn latest_gmma_store(
    store: &CandleStore,
    outputs: &crate::types::IndicatorArena,
) -> Vec<(String, Option<f64>)> {
    let mut results = Vec::with_capacity(12);
    for (&p, &slot) in SHORT_PERIODS.iter().zip(SHORT_SLOTS.iter()) {
        let name = output_name("short", p);
        let val = latest_ema_store(store, p, outputs.get_slot(slot));
        results.push((name, val));
    }
    for (&p, &slot) in LONG_PERIODS.iter().zip(LONG_SLOTS.iter()) {
        let name = output_name("long", p);
        let val = latest_ema_store(store, p, outputs.get_slot(slot));
        results.push((name, val));
    }
    results
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
    fn gmma_is_flat_for_constant_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let outputs = gmma_store(&store, &mut HashMap::new());
        for series in &outputs {
            assert_eq!(&*series.values, &[10.0, 10.0, 10.0, 10.0]);
        }

        let arena = IndicatorArena::from_named_outputs(outputs);
        let latest = latest_gmma_store(&store, &arena);
        assert_eq!(latest.len(), 12);
        for (_, value) in latest {
            assert_eq!(value, Some(10.0));
        }
    }

    #[test]
    fn gmma_exposes_each_expected_ema_series() {
        let store = close_store(&(1..=60).map(|v| v as f64).collect::<Vec<_>>());
        let outputs = gmma_store(&store, &mut HashMap::new());

        assert_eq!(outputs[0].values[59], 59.0);
        assert!((outputs[5].values[59] - 53.00265199785418).abs() < 1e-12);
        assert!((outputs[11].values[59] - 34.62696168835945).abs() < 1e-12);

        let arena = IndicatorArena::from_named_outputs(outputs);
        let latest = latest_gmma_store(&store, &arena);
        assert_eq!(latest[0], ("short_3".to_string(), Some(59.0)));
        assert_eq!(latest[5], ("short_15".to_string(), Some(53.00265199785418)));
        assert_eq!(latest[11], ("long_60".to_string(), Some(34.62696168835945)));
    }
}

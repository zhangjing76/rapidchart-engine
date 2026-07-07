use crate::NodeCache;
use crate::CandleStore;
use crate::indicators::ema::{ema_close_store, latest_ema_store};

/// GMMA periods: short-term group and long-term group.
const SHORT_PERIODS: [usize; 6] = [3, 5, 8, 10, 12, 15];
const LONG_PERIODS: [usize; 6] = [30, 35, 40, 45, 50, 60];

fn output_name(prefix: &str, period: usize) -> String {
    format!("{prefix}_{period}")
}

/// Guppy Multiple Moving Average: 12 EMA lines.

pub fn gmma_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    let mut outputs = Vec::with_capacity(12);
    for &p in &SHORT_PERIODS {
        outputs.push(crate::named_series(output_name("short", p), ema_close_store(store, p, nodes)));
    }
    for &p in &LONG_PERIODS {
        outputs.push(crate::named_series(output_name("long", p), ema_close_store(store, p, nodes)));
    }
    outputs
}

pub fn latest_gmma_store(
    store: &CandleStore,
    outputs: &crate::types::IndicatorArena,
) -> Vec<(String, Option<f64>)> {
    let mut results = Vec::with_capacity(12);
    for &p in &SHORT_PERIODS {
        let name = output_name("short", p);
        let val = latest_ema_store(store, p, outputs.get(&name));
        results.push((name, val));
    }
    for &p in &LONG_PERIODS {
        let name = output_name("long", p);
        let val = latest_ema_store(store, p, outputs.get(&name));
        results.push((name, val));
    }
    results
}

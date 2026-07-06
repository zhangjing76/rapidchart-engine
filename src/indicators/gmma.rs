use crate::NodeCache;
use crate::{Bar, CandleStore, IndicatorOutput};
use crate::indicators::ema::{ema_close, ema_close_store, latest_ema_store};
use crate::series::rc_into_owned;

/// GMMA periods: short-term group and long-term group.
const SHORT_PERIODS: [usize; 6] = [3, 5, 8, 10, 12, 15];
const LONG_PERIODS: [usize; 6] = [30, 35, 40, 45, 50, 60];

fn output_name(prefix: &str, period: usize) -> String {
    format!("{prefix}_{period}")
}

/// Guppy Multiple Moving Average: 12 EMA lines.
pub fn gmma(bars: &[Bar], nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let mut outputs = Vec::with_capacity(12);
    for &p in &SHORT_PERIODS {
        let values = ema_close(bars, p, nodes);
        outputs.push(IndicatorOutput {
            name: output_name("short", p),
            values,
        });
    }
    for &p in &LONG_PERIODS {
        let values = ema_close(bars, p, nodes);
        outputs.push(IndicatorOutput {
            name: output_name("long", p),
            values,
        });
    }
    outputs
}

pub fn gmma_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let mut outputs = Vec::with_capacity(12);
    for &p in &SHORT_PERIODS {
        let values = rc_into_owned(ema_close_store(store, p, nodes));
        outputs.push(IndicatorOutput {
            name: output_name("short", p),
            values,
        });
    }
    for &p in &LONG_PERIODS {
        let values = rc_into_owned(ema_close_store(store, p, nodes));
        outputs.push(IndicatorOutput {
            name: output_name("long", p),
            values,
        });
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

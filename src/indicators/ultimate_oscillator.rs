use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

pub fn ultimate_oscillator(bars: &[Bar], short: usize, medium: usize, long: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if short == 0 || medium == 0 || long == 0 || bars.len() <= long {
        return out;
    }
    let mut bp = vec![0.0; bars.len()];
    let mut tr = vec![0.0; bars.len()];
    for index in 1..bars.len() {
        let previous_close = bars[index - 1].close;
        let min_low = bars[index].low.min(previous_close);
        let max_high = bars[index].high.max(previous_close);
        bp[index] = bars[index].close - min_low;
        tr[index] = max_high - min_low;
    }
    for index in long..bars.len() {
        let avg = |period: usize| {
            let start = index + 1 - period;
            let bp_sum = bp[start..=index].iter().sum::<f64>();
            let tr_sum = tr[start..=index].iter().sum::<f64>();
            if tr_sum == 0.0 {
                0.0
            } else {
                bp_sum / tr_sum
            }
        };
        out[index] = 100.0 * (4.0 * avg(short) + 2.0 * avg(medium) + avg(long)) / 7.0;
    }
    out
}
pub fn ultimate_oscillator_node(
    bars: &[Bar],
    short: usize,
    medium: usize,
    long: usize,
    nodes: &mut NodeCache,
) -> Series {
    let key = format!("uo:{short}:{medium}:{long}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = ultimate_oscillator(bars, short, medium, long);
    nodes.insert(key, Rc::new(values.clone()));
    values
}
pub fn ultimate_oscillator_store(
    store: &CandleStore,
    short: usize,
    medium: usize,
    long: usize,
    nodes: &mut NodeCache,
) -> Series {
    let key = format!("uo:{short}:{medium}:{long}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let mut out = vec![f64::NAN; store.len()];
    if short == 0 || medium == 0 || long == 0 || store.len() <= long {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    let mut bp = vec![0.0; store.len()];
    let mut tr = vec![0.0; store.len()];
    for index in 1..store.len() {
        let previous_close = store.close[index - 1];
        let min_low = store.low[index].min(previous_close);
        let max_high = store.high[index].max(previous_close);
        bp[index] = store.close[index] - min_low;
        tr[index] = max_high - min_low;
    }
    for index in long..store.len() {
        let avg = |period: usize| {
            let start = index + 1 - period;
            let bp_sum = bp[start..=index].iter().sum::<f64>();
            let tr_sum = tr[start..=index].iter().sum::<f64>();
            if tr_sum == 0.0 {
                0.0
            } else {
                bp_sum / tr_sum
            }
        };
        out[index] = 100.0 * (4.0 * avg(short) + 2.0 * avg(medium) + avg(long)) / 7.0;
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}
pub fn latest_ultimate_oscillator(
    bars: &[Bar],
    short: usize,
    medium: usize,
    long: usize,
) -> Option<f64> {
    ultimate_oscillator(bars, short, medium, long)
        .last()
        .copied()
        .and_then(nan_to_none)
}
pub fn latest_ultimate_oscillator_store(
    store: &CandleStore,
    short: usize,
    medium: usize,
    long: usize,
) -> Option<f64> {
    ultimate_oscillator_store(store, short, medium, long, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

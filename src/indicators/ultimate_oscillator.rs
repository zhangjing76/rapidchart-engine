use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::collections::HashMap;
use std::rc::Rc;

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
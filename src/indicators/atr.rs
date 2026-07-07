use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

pub fn true_range_store(store: &CandleStore, index: usize) -> f64 {
    if index == 0 {
        return store.high[0] - store.low[0];
    }
    let previous_close = store.close[index - 1];
    (store.high[index] - store.low[index])
        .max((store.high[index] - previous_close).abs())
        .max((store.low[index] - previous_close).abs())
}
pub fn atr_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("atr:ohlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let mut current = (1..=period)
        .map(|index| true_range_store(store, index))
        .sum::<f64>()
        / period as f64;
    out[period] = current;
    for (index, item) in out.iter_mut().enumerate().skip(period + 1) {
        current = (current * (period - 1) as f64 + true_range_store(store, index)) / period as f64;
        *item = current;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_atr_store(store: &CandleStore, period: usize, output: Option<&[f64]>) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    if store.len() == period + 1 {
        return atr_store(store, period, &mut HashMap::new())
            .last()
            .copied()
            .and_then(nan_to_none);
    }
    let previous_index = store.len() - 2;
    let previous = output
        .and_then(|values| values.get(previous_index))
        .copied()
        .and_then(nan_to_none)
        .unwrap_or_else(|| {
            let previous = CandleStore {
                time: store.time[..store.len() - 1].to_vec(),
                open: store.open[..store.len() - 1].to_vec(),
                high: store.high[..store.len() - 1].to_vec(),
                low: store.low[..store.len() - 1].to_vec(),
                close: store.close[..store.len() - 1].to_vec(),
                volume: store.volume[..store.len() - 1].to_vec(),
            };
            atr_store(&previous, period, &mut HashMap::new())[previous_index]
        });
    Some(
        (previous * (period - 1) as f64 + true_range_store(store, store.len() - 1)) / period as f64,
    )
}

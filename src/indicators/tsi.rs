use crate::indicators::ema::ema_series;
use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

pub fn tsi_store(
    store: &CandleStore,
    long: usize,
    short: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("tsi:{long}:{short}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut momentum = vec![f64::NAN; store.len()];
    let mut abs_momentum = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        let value = store.close[index] - store.close[index - 1];
        momentum[index] = value;
        abs_momentum[index] = value.abs();
    }
    let ema1 = ema_series(&momentum, long);
    let ema2 = ema_series(&ema1, short);
    let abs_ema1 = ema_series(&abs_momentum, long);
    let abs_ema2 = ema_series(&abs_ema1, short);
    let values: Series = ema2
        .iter()
        .zip(abs_ema2.iter())
        .map(|(num, den)| match (num, den) {
            (num, den) if !num.is_nan() && !den.is_nan() && *den != 0.0 => 100.0 * *num / *den,
            (a, b) if !a.is_nan() && !b.is_nan() => 0.0,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_tsi_store(store: &CandleStore, long: usize, short: usize) -> Option<f64> {
    tsi_store(store, long, short, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

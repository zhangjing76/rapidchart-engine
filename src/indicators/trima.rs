use crate::indicators::kst::sma_from_series;
use crate::indicators::sma::sma_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn trima_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("trima:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let values = sma_from_series(&sma_close_store(store, period, nodes), period);
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_trima_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    // TRIMA = SMA of SMA(close, period), period
    // We need at least 2*period - 1 bars for the last SMA(SMA) value to be valid
    let needed = 2 * period - 1;
    if store.len() < needed {
        return None;
    }
    // Compute SMA values for the last `period` positions
    let p = period as f64;
    let start = store.len() - needed;
    let mut sma_values = Vec::with_capacity(period);
    for i in (start + period - 1)..store.len() {
        let window = &store.close[i + 1 - period..=i];
        sma_values.push(window.iter().sum::<f64>() / p);
    }
    if sma_values.len() < period {
        return None;
    }
    // SMA of the last `period` SMA values
    let last_period = &sma_values[sma_values.len() - period..];
    Some(last_period.iter().sum::<f64>() / p)
}

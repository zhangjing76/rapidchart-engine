use crate::indicators::kst::sma_from_series;
use crate::indicators::sma::{sma_close, sma_close_store, sma_close_values};
use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

pub fn trima(bars: &[Bar], period: usize) -> Series {
    sma_from_series(&sma_close(bars, period, &mut HashMap::new()), period)
}
pub fn trima_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("trima:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = sma_from_series(&sma_close(bars, period, nodes), period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}
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
pub fn latest_trima(bars: &[Bar], period: usize) -> Option<f64> {
    trima(bars, period).last().copied().and_then(nan_to_none)
}
pub fn latest_trima_store(store: &CandleStore, period: usize) -> Option<f64> {
    trima_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

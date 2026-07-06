use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

/// Median Price = (High + Low) / 2
pub fn median_price(bars: &[Bar]) -> Series {
    bars.iter().map(|bar| (bar.high + bar.low) / 2.0).collect()
}

pub fn median_price_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "median_price:hl".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = median_price(bars);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

pub fn median_price_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "median_price:hl".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(h, l)| (h + l) / 2.0)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

#[allow(dead_code)]
pub fn latest_median_price(bars: &[Bar]) -> Option<f64> {
    let bar = bars.last()?;
    Some((bar.high + bar.low) / 2.0)
}

pub fn latest_median_price_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 {
        return None;
    }
    let i = store.len() - 1;
    Some((store.high[i] + store.low[i]) / 2.0)
}

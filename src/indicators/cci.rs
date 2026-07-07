use crate::indicators::derived::{hlc3_store, latest_hlc3};
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Typical price used by CCI, VWAP, and MFI.
pub fn typical_price_parts(high: f64, low: f64, close: f64) -> f64 {
    (high + low + close) / 3.0
}
/// Typical price at a given bar index.
pub fn typical_price_at(store: &CandleStore, index: usize) -> f64 {
    typical_price_parts(store.high[index], store.low[index], store.close[index])
}

pub fn cci_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("cci:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let hlc3 = hlc3_store(store, nodes);
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
        let window = index + 1 - period..=index;
        let typical_prices: Vec<_> = window.clone().map(|i| hlc3[i]).collect();
        let sma = typical_prices.iter().sum::<f64>() / period as f64;
        let mean_deviation = typical_prices
            .iter()
            .map(|value| (value - sma).abs())
            .sum::<f64>()
            / period as f64;
        *item = if mean_deviation == 0.0 {
            0.0
        } else {
            (hlc3[index] - sma) / (0.015 * mean_deviation)
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_cci_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let hlc3_last = latest_hlc3(store)?;
    let typical_prices: Vec<_> = (start..store.len())
        .map(|i| (store.high[i] + store.low[i] + store.close[i]) / 3.0)
        .collect();
    let sma = typical_prices.iter().sum::<f64>() / period as f64;
    let mean_deviation = typical_prices
        .iter()
        .map(|value| (value - sma).abs())
        .sum::<f64>()
        / period as f64;
    Some(if mean_deviation == 0.0 {
        0.0
    } else {
        (hlc3_last - sma) / (0.015 * mean_deviation)
    })
}

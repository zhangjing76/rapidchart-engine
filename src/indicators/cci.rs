use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn cci(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let typical_prices: Vec<_> = window.iter().map(typical_price).collect();
        let sma = typical_prices.iter().sum::<f64>() / period as f64;
        let mean_deviation = typical_prices
            .iter()
            .map(|value| (value - sma).abs())
            .sum::<f64>()
            / period as f64;
        out[index] = if mean_deviation == 0.0 {
            0.0
        } else {
            (typical_price(&bars[index]) - sma) / (0.015 * mean_deviation)
        };
    }
    out
}
pub fn typical_price(bar: &Bar) -> f64 {
    typical_price_parts(bar.high, bar.low, bar.close)
}
pub fn typical_price_parts(high: f64, low: f64, close: f64) -> f64 {
    (high + low + close) / 3.0
}
pub fn typical_price_at(store: &CandleStore, index: usize) -> f64 {
    typical_price_parts(store.high[index], store.low[index], store.close[index])
}
pub fn cci_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("cci:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = cci(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
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
    for index in period - 1..store.len() {
        let window = index + 1 - period..=index;
        let typical_prices: Vec<_> = window.clone().map(|i| typical_price_at(store, i)).collect();
        let sma = typical_prices.iter().sum::<f64>() / period as f64;
        let mean_deviation = typical_prices
            .iter()
            .map(|value| (value - sma).abs())
            .sum::<f64>()
            / period as f64;
        out[index] = if mean_deviation == 0.0 {
            0.0
        } else {
            (typical_price_at(store, index) - sma) / (0.015 * mean_deviation)
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_cci(bars: &[Bar], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    let window = &bars[bars.len() - period..];
    let typical_prices: Vec<_> = window.iter().map(typical_price).collect();
    let sma = typical_prices.iter().sum::<f64>() / period as f64;
    let mean_deviation = typical_prices
        .iter()
        .map(|value| (value - sma).abs())
        .sum::<f64>()
        / period as f64;
    Some(if mean_deviation == 0.0 {
        0.0
    } else {
        (typical_price(bars.last().expect("checked non-empty")) - sma) / (0.015 * mean_deviation)
    })
}
pub fn latest_cci_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let typical_prices: Vec<_> = (start..store.len())
        .map(|index| typical_price_at(store, index))
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
        (typical_price_at(store, store.len() - 1) - sma) / (0.015 * mean_deviation)
    })
}

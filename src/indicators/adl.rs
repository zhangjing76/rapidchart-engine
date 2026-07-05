use crate::nan_to_none;
use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn adl(bars: &[Bar]) -> Series {
    let mut out = Vec::with_capacity(bars.len());
    let mut current = 0.0;
    for bar in bars {
        current += money_flow_multiplier(bar) * bar.volume;
        out.push(current);
    }
    out
}
pub fn money_flow_multiplier(bar: &Bar) -> f64 {
    money_flow_multiplier_parts(bar.high, bar.low, bar.close)
}
pub fn money_flow_multiplier_parts(high: f64, low: f64, close: f64) -> f64 {
    let range = high - low;
    if range == 0.0 {
        0.0
    } else {
        ((close - low) - (high - close)) / range
    }
}
pub fn adl_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "adl:hlcv".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = adl(bars);
    nodes.insert(key, Rc::new(values.clone()));
    values
}
pub fn adl_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "adl:hlcv".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = Vec::with_capacity(store.len());
    let mut current = 0.0;
    for (((&high, &low), &close), &volume) in store
        .high
        .iter()
        .zip(store.low.iter())
        .zip(store.close.iter())
        .zip(store.volume.iter())
    {
        current += money_flow_multiplier_parts(high, low, close) * volume;
        out.push(current);
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
#[allow(dead_code)]
pub fn latest_adl(bars: &[Bar], output: Option<&[f64]>) -> Option<f64> {
    let last = bars.last()?;
    let previous = bars
        .len()
        .checked_sub(2)
        .and_then(|index| {
            output
                .and_then(|values| values.get(index))
                .copied()
                .and_then(nan_to_none)
        })
        .unwrap_or(0.0);
    Some(previous + money_flow_multiplier(last) * last.volume)
}
pub fn latest_adl_store(store: &CandleStore, output: Option<&[f64]>) -> Option<f64> {
    let index = store.len().checked_sub(1)?;
    let previous = index
        .checked_sub(1)
        .and_then(|previous_index| {
            output
                .and_then(|values| values.get(previous_index))
                .copied()
                .and_then(nan_to_none)
        })
        .unwrap_or(0.0);
    Some(
        previous
            + money_flow_multiplier_parts(store.high[index], store.low[index], store.close[index])
                * store.volume[index],
    )
}

use crate::nan_to_none;
use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

#[allow(dead_code)]
pub fn obv(bars: &[Bar]) -> Series {
    let mut out = Vec::with_capacity(bars.len());
    let mut current = 0.0;
    for (i, bar) in bars.iter().enumerate() {
        if i > 0 {
            let previous = bars[i - 1].close;
            if bar.close > previous {
                current += bar.volume;
            } else if bar.close < previous {
                current -= bar.volume;
            }
        }
        out.push(current);
    }
    out
}
#[allow(dead_code)]
pub fn obv_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "obv:close:volume".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = obv(bars);
    nodes.insert(key, Rc::new(values.clone()));
    values
}
pub fn obv_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "obv:close:volume".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = Vec::with_capacity(store.len());
    let mut current = 0.0;
    for (index, (&close, &volume)) in store.close.iter().zip(store.volume.iter()).enumerate() {
        if index > 0 {
            let previous = store.close[index - 1];
            if close > previous {
                current += volume;
            } else if close < previous {
                current -= volume;
            }
        }
        out.push(current);
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
#[allow(dead_code)]
pub fn latest_obv(bars: &[Bar], output: Option<&[f64]>) -> Option<f64> {
    let last = bars.last()?;
    if bars.len() == 1 {
        return Some(0.0);
    }
    let previous = output
        .and_then(|values| values.get(bars.len() - 2))
        .copied()
        .and_then(nan_to_none)
        .unwrap_or(0.0);
    let previous_close = bars[bars.len() - 2].close;
    if last.close > previous_close {
        Some(previous + last.volume)
    } else if last.close < previous_close {
        Some(previous - last.volume)
    } else {
        Some(previous)
    }
}
pub fn latest_obv_store(store: &CandleStore, output: Option<&[f64]>) -> Option<f64> {
    let last = store.last_close()?;
    if store.len() == 1 {
        return Some(0.0);
    }
    let previous = output
        .and_then(|values| values.get(store.len() - 2))
        .copied()
        .and_then(nan_to_none)
        .unwrap_or(0.0);
    let previous_close = store.close[store.len() - 2];
    if last > previous_close {
        Some(previous + store.volume[store.len() - 1])
    } else if last < previous_close {
        Some(previous - store.volume[store.len() - 1])
    } else {
        Some(previous)
    }
}

use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use crate::indicators::sma::sma_close_store;
use crate::indicators::atr::atr_store;
use std::collections::HashMap;
use std::rc::Rc;

/// Pretty Good Oscillator: (close - SMA(close, period)) / ATR(period)
pub fn pretty_good_oscillator_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("pgo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma = sma_close_store(store, period, nodes);
    let atr = atr_store(store, period, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let s = sma[i];
        let a = atr[i];
        if !s.is_nan() && !a.is_nan() && a.abs() > 1e-10 {
            out[i] = (store.close[i] - s) / a;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn pretty_good_oscillator_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("pgo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let sma = crate::indicators::sma::sma_close(bars, period, nodes);
    let atr = crate::indicators::atr::atr_node(bars, period, nodes);
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let s = sma[i];
        let a = atr[i];
        if !s.is_nan() && !a.is_nan() && a.abs() > 1e-10 {
            out[i] = (bars[i].close - s) / a;
        }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_pretty_good_oscillator_store(store: &CandleStore, period: usize) -> Option<f64> {
    pretty_good_oscillator_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

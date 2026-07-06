use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use crate::indicators::sma::sma_close_store;
use std::collections::HashMap;
use std::rc::Rc;

/// Trend Intensity Index:
/// Counts bars above SMA vs below SMA over period.
/// TII = (bars_above - bars_below) / period * 100
/// Range: -100 (all below) to +100 (all above)
pub fn trend_intensity_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("tii:close:{period}");
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let sma = sma_close_store(store, period, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period { 
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..len {
        let sma_val = sma[i];
        if sma_val.is_nan() { continue; }
        let mut above = 0i32;
        let mut below = 0i32;
        for j in i + 1 - period..=i {
            if store.close[j] > sma_val { above += 1; }
            else { below += 1; }
        }
        out[i] = ((above - below) as f64 / period as f64) * 100.0;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn trend_intensity_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("tii:close:{period}");
    if let Some(values) = nodes.get(&key) { return (**values).clone(); }
    let sma = crate::indicators::sma::sma_close(bars, period, nodes);
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    for i in period - 1..len {
        let sma_val = sma[i];
        if sma_val.is_nan() { continue; }
        let mut above = 0i32;
        let mut below = 0i32;
        for j in i + 1 - period..=i {
            if bars[j].close > sma_val { above += 1; } else { below += 1; }
        }
        out[i] = ((above - below) as f64 / period as f64) * 100.0;
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_trend_intensity_store(store: &CandleStore, period: usize) -> Option<f64> {
    trend_intensity_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use crate::indicators::ema::ema_series;
use std::collections::HashMap;
use std::rc::Rc;

/// Klinger Volume Oscillator:
/// Volume Force (VF) = volume * |2*(dm/cm) - 1| * trend * 100
/// where trend = +1 if (H+L+C) > prev(H+L+C), else -1
/// dm = high - low, cm = cumulative dm in same trend direction
/// KVO = EMA(34, VF) - EMA(55, VF)
pub fn klinger_volume_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "klinger:hlcv".to_string();
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let mut vf = vec![0.0f64; len];
    let mut cm = 0.0f64;
    let mut prev_trend = 1i8;
    for i in 1..len {
        let hlc = store.high[i] + store.low[i] + store.close[i];
        let prev_hlc = store.high[i-1] + store.low[i-1] + store.close[i-1];
        let trend: i8 = if hlc > prev_hlc { 1 } else { -1 };
        let dm = store.high[i] - store.low[i];
        if trend == prev_trend {
            cm += dm;
        } else {
            cm = dm;
        }
        let ratio = if cm.abs() > 1e-10 { (2.0 * dm / cm) - 1.0 } else { 0.0 };
        vf[i] = store.volume[i] * ratio.abs() * trend as f64 * 100.0;
        prev_trend = trend;
    }
    let ema34 = ema_series(&vf, 34);
    let ema55 = ema_series(&vf, 55);
    for i in 0..len {
        if !ema34[i].is_nan() && !ema55[i].is_nan() {
            out[i] = ema34[i] - ema55[i];
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn klinger_volume_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "klinger:hlcv".to_string();
    if let Some(values) = nodes.get(&key) { return (**values).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 { nodes.insert(key, Rc::new(out.clone())); return out; }
    let mut vf = vec![0.0f64; len];
    let mut cm = 0.0f64;
    let mut prev_trend = 1i8;
    for i in 1..len {
        let hlc = bars[i].high + bars[i].low + bars[i].close;
        let prev_hlc = bars[i-1].high + bars[i-1].low + bars[i-1].close;
        let trend: i8 = if hlc > prev_hlc { 1 } else { -1 };
        let dm = bars[i].high - bars[i].low;
        if trend == prev_trend { cm += dm; } else { cm = dm; }
        let ratio = if cm.abs() > 1e-10 { (2.0 * dm / cm) - 1.0 } else { 0.0 };
        vf[i] = bars[i].volume * ratio.abs() * trend as f64 * 100.0;
        prev_trend = trend;
    }
    let ema34 = ema_series(&vf, 34);
    let ema55 = ema_series(&vf, 55);
    for i in 0..len {
        if !ema34[i].is_nan() && !ema55[i].is_nan() { out[i] = ema34[i] - ema55[i]; }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_klinger_volume_store(store: &CandleStore) -> Option<f64> {
    klinger_volume_store(store, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

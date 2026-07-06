use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use crate::indicators::ema::ema_series;
use std::collections::HashMap;
use std::rc::Rc;

/// Stochastic Momentum Index (SMI):
/// D = close - (highest_high + lowest_low) / 2 over period
/// SMI = 100 * EMA(EMA(D, smooth), smooth) / EMA(EMA(HL_range/2, smooth), smooth)
pub fn stochastic_momentum_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("smi:hlc:{}:{}", period, smooth);
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let mut d_series = vec![f64::NAN; len];
    let mut hl_series = vec![f64::NAN; len];
    for i in period - 1..len {
        let hh = store.high[i+1-period..=i].iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = store.low[i+1-period..=i].iter().fold(f64::INFINITY, |a, &b| a.min(b));
        d_series[i] = store.close[i] - (hh + ll) / 2.0;
        hl_series[i] = (hh - ll) / 2.0;
    }
    let d_smooth1 = ema_series(&d_series, smooth);
    let d_smooth2 = ema_series(&d_smooth1, smooth);
    let hl_smooth1 = ema_series(&hl_series, smooth);
    let hl_smooth2 = ema_series(&hl_smooth1, smooth);
    for i in 0..len {
        let d = d_smooth2[i];
        let hl = hl_smooth2[i];
        if !d.is_nan() && !hl.is_nan() && hl.abs() > 1e-10 {
            out[i] = (d / hl) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn stochastic_momentum_node(bars: &[Bar], period: usize, smooth: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("smi:hlc:{}:{}", period, smooth);
    if let Some(values) = nodes.get(&key) { return (**values).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    let mut d_series = vec![f64::NAN; len];
    let mut hl_series = vec![f64::NAN; len];
    for i in period - 1..len {
        let hh = bars[i+1-period..=i].iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let ll = bars[i+1-period..=i].iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        d_series[i] = bars[i].close - (hh + ll) / 2.0;
        hl_series[i] = (hh - ll) / 2.0;
    }
    let d_smooth1 = ema_series(&d_series, smooth);
    let d_smooth2 = ema_series(&d_smooth1, smooth);
    let hl_smooth1 = ema_series(&hl_series, smooth);
    let hl_smooth2 = ema_series(&hl_smooth1, smooth);
    for i in 0..len {
        let d = d_smooth2[i];
        let hl = hl_smooth2[i];
        if !d.is_nan() && !hl.is_nan() && hl.abs() > 1e-10 {
            out[i] = (d / hl) * 100.0;
        }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_stochastic_momentum_store(store: &CandleStore, period: usize, smooth: usize) -> Option<f64> {
    stochastic_momentum_store(store, period, smooth, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

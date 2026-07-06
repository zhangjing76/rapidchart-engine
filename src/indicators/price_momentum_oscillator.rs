use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use crate::indicators::ema::ema_series;
use std::collections::HashMap;
use std::rc::Rc;

/// Price Momentum Oscillator (PMO): Double-smoothed ROC.
/// 1. ROC1 = ((close / close[1]) - 1) * 100
/// 2. Smooth1 = EMA(ROC1, period) — first smoothing (default period=35)
/// 3. PMO = EMA(Smooth1 * 10, smooth) — second smoothing (default smooth=20)
pub fn price_momentum_oscillator_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("pmo:close:{}:{}", period, smooth);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // Step 1: ROC(1) percentage
    let mut roc1 = vec![f64::NAN; len];
    for i in 1..len {
        if store.close[i - 1] != 0.0 {
            roc1[i] = ((store.close[i] / store.close[i - 1]) - 1.0) * 100.0;
        }
    }
    // Step 2: EMA of ROC1 with period
    let smooth1 = ema_series(&roc1, period);
    // Step 3: Multiply by 10 and EMA again with smooth
    let scaled: Vec<f64> = smooth1.iter().map(|&v| if v.is_nan() { f64::NAN } else { v * 10.0 }).collect();
    let pmo = ema_series(&scaled, smooth);
    out = pmo;
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn price_momentum_oscillator_node(
    bars: &[Bar],
    period: usize,
    smooth: usize,
    nodes: &mut NodeCache,
) -> Series {
    let key = format!("pmo:close:{}:{}", period, smooth);
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let len = bars.len();
    if len < 2 {
        let out = vec![f64::NAN; len];
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    let mut roc1 = vec![f64::NAN; len];
    for i in 1..len {
        if bars[i - 1].close != 0.0 {
            roc1[i] = ((bars[i].close / bars[i - 1].close) - 1.0) * 100.0;
        }
    }
    let smooth1 = ema_series(&roc1, period);
    let scaled: Vec<f64> = smooth1.iter().map(|&v| if v.is_nan() { f64::NAN } else { v * 10.0 }).collect();
    let out = ema_series(&scaled, smooth);
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_price_momentum_oscillator_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
) -> Option<f64> {
    price_momentum_oscillator_store(store, period, smooth, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

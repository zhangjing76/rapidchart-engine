use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Coppock Curve: WMA(10) of (ROC(14) + ROC(11))
/// A momentum indicator originally designed for monthly charts.
pub fn coppock_curve_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "coppock:close".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    // Need at least 14 bars for ROC(14)
    if len < 15 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // Compute ROC(14) + ROC(11) series
    let mut roc_sum = vec![f64::NAN; len];
    for i in 14..len {
        let roc14 = if store.close[i - 14] != 0.0 {
            ((store.close[i] - store.close[i - 14]) / store.close[i - 14]) * 100.0
        } else { f64::NAN };
        let roc11 = if i >= 11 && store.close[i - 11] != 0.0 {
            ((store.close[i] - store.close[i - 11]) / store.close[i - 11]) * 100.0
        } else { f64::NAN };
        if !roc14.is_nan() && !roc11.is_nan() {
            roc_sum[i] = roc14 + roc11;
        }
    }
    // WMA(10) of roc_sum
    let wma_period = 10;
    let weight_sum: f64 = (1..=wma_period).map(|w| w as f64).sum();
    for i in 0..len {
        if i + 1 < wma_period { continue; }
        let window = &roc_sum[i + 1 - wma_period..=i];
        if window.iter().any(|v| v.is_nan()) { continue; }
        let weighted: f64 = window.iter().enumerate()
            .map(|(j, &v)| v * (j + 1) as f64).sum();
        out[i] = weighted / weight_sum;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn coppock_curve_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "coppock:close".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if len < 15 {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    let mut roc_sum = vec![f64::NAN; len];
    for i in 14..len {
        let roc14 = if bars[i - 14].close != 0.0 {
            ((bars[i].close - bars[i - 14].close) / bars[i - 14].close) * 100.0
        } else { f64::NAN };
        let roc11 = if i >= 11 && bars[i - 11].close != 0.0 {
            ((bars[i].close - bars[i - 11].close) / bars[i - 11].close) * 100.0
        } else { f64::NAN };
        if !roc14.is_nan() && !roc11.is_nan() {
            roc_sum[i] = roc14 + roc11;
        }
    }
    let wma_period = 10;
    let weight_sum: f64 = (1..=wma_period).map(|w| w as f64).sum();
    for i in 0..len {
        if i + 1 < wma_period { continue; }
        let window = &roc_sum[i + 1 - wma_period..=i];
        if window.iter().any(|v| v.is_nan()) { continue; }
        let weighted: f64 = window.iter().enumerate()
            .map(|(j, &v)| v * (j + 1) as f64).sum();
        out[i] = weighted / weight_sum;
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_coppock_curve_store(store: &CandleStore) -> Option<f64> {
    coppock_curve_store(store, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

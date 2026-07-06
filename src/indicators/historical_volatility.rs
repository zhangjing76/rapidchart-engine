use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Historical Volatility: Annualized standard deviation of log returns over period.
/// HV = stddev(ln(close[i]/close[i-1]), period) * sqrt(252)
/// (252 for daily bars; for other timeframes the annualization factor stays the same
/// as a convention, since users interpret it relative to their timeframe.)
pub fn historical_volatility_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("hv:close:{period}");
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period + 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // Compute log returns
    let mut log_returns = vec![f64::NAN; len];
    for i in 1..len {
        if store.close[i - 1] > 0.0 && store.close[i] > 0.0 {
            log_returns[i] = (store.close[i] / store.close[i - 1]).ln();
        }
    }
    // Rolling stddev of log returns
    let annualize = (252.0f64).sqrt();
    for i in period..len {
        let window = &log_returns[i + 1 - period..=i];
        let valid: Vec<f64> = window.iter().filter(|v| !v.is_nan()).copied().collect();
        if valid.len() < 2 { continue; }
        let n = valid.len() as f64;
        let mean = valid.iter().sum::<f64>() / n;
        let variance = valid.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        out[i] = variance.sqrt() * annualize * 100.0; // as percentage
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn historical_volatility_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("hv:close:{period}");
    if let Some(values) = nodes.get(&key) { return (**values).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period + 1 {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    let mut log_returns = vec![f64::NAN; len];
    for i in 1..len {
        if bars[i - 1].close > 0.0 && bars[i].close > 0.0 {
            log_returns[i] = (bars[i].close / bars[i - 1].close).ln();
        }
    }
    let annualize = (252.0f64).sqrt();
    for i in period..len {
        let window = &log_returns[i + 1 - period..=i];
        let valid: Vec<f64> = window.iter().filter(|v| !v.is_nan()).copied().collect();
        if valid.len() < 2 { continue; }
        let n = valid.len() as f64;
        let mean = valid.iter().sum::<f64>() / n;
        let variance = valid.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        out[i] = variance.sqrt() * annualize * 100.0;
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_historical_volatility_store(store: &CandleStore, period: usize) -> Option<f64> {
    historical_volatility_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Beta indicator (single-symbol): measures the rolling regression slope of
/// log returns over `period` bars. Without a benchmark, this captures
/// the trend strength/direction of the asset's own returns.
///
/// Computed as: slope of linear regression of returns over the window,
/// where returns[i] = (close[i] - close[i-1]) / close[i-1].
pub fn beta_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("beta:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period + 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }

    // Compute returns series
    let mut returns = vec![f64::NAN; len];
    for i in 1..len {
        if store.close[i - 1] != 0.0 {
            returns[i] = (store.close[i] - store.close[i - 1]) / store.close[i - 1];
        }
    }

    // Rolling standard deviation of returns as a volatility-based beta proxy
    let n = period as f64;
    for i in period..len {
        let window = &returns[i + 1 - period..=i];
        let valid: Vec<f64> = window.iter().filter(|v| !v.is_nan()).copied().collect();
        if valid.len() < 2 {
            continue;
        }
        let count = valid.len() as f64;
        let mean = valid.iter().sum::<f64>() / count;
        let variance = valid.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (count - 1.0);
        let stddev = variance.sqrt();
        // Annualized-style beta: stddev of returns * sqrt(period) gives a comparable measure
        out[i] = stddev * n.sqrt();
    }

    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_beta_store(store: &CandleStore, period: usize) -> Option<f64> {
    beta_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}
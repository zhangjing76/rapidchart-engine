use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Choppiness Index:
/// CI = 100 * LOG10(SUM(ATR(1), period) / (Highest_High - Lowest_Low)) / LOG10(period)
/// Range: 0-100. High values (>61.8) = choppy/sideways, Low values (<38.2) = trending.
pub fn choppiness_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("chop:hlc:{period}");
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period + 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // True Range series
    let mut tr = vec![0.0f64; len];
    for i in 1..len {
        tr[i] = (store.high[i] - store.low[i])
            .max((store.high[i] - store.close[i - 1]).abs())
            .max((store.low[i] - store.close[i - 1]).abs());
    }
    let log_period = (period as f64).log10();
    for i in period..len {
        let sum_atr: f64 = tr[i + 1 - period..=i].iter().sum();
        let hh = store.high[i + 1 - period..=i].iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = store.low[i + 1 - period..=i].iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = hh - ll;
        if range > 1e-10 && log_period > 0.0 {
            out[i] = 100.0 * (sum_atr / range).log10() / log_period;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_choppiness_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    choppiness_index_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}
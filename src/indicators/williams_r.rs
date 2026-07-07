use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn williams_r_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("willr:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
        let window = index + 1 - period..=index;
        let highest_high = window
            .clone()
            .map(|i| store.high[i])
            .fold(f64::MIN, f64::max);
        let lowest_low = window.map(|i| store.low[i]).fold(f64::MAX, f64::min);
        let range = highest_high - lowest_low;
        *item = if range == 0.0 {
            0.0
        } else {
            -100.0 * (highest_high - store.close[index]) / range
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_williams_r_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let highest_high = store.high[start..].iter().copied().fold(f64::MIN, f64::max);
    let lowest_low = store.low[start..].iter().copied().fold(f64::MAX, f64::min);
    let range = highest_high - lowest_low;
    Some(if range == 0.0 {
        0.0
    } else {
        -100.0 * (highest_high - store.close[store.len() - 1]) / range
    })
}

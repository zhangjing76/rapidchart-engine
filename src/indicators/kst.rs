use crate::indicators::roc::roc_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn kst_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "kst:value".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let roc1 = roc_store(store, 10, nodes);
    let roc2 = roc_store(store, 15, nodes);
    let roc3 = roc_store(store, 20, nodes);
    let roc4 = roc_store(store, 30, nodes);
    let sma1 = sma_from_series(&roc1, 10);
    let sma2 = sma_from_series(&roc2, 10);
    let sma3 = sma_from_series(&roc3, 10);
    let sma4 = sma_from_series(&roc4, 15);
    let values: Vec<_> = sma1
        .iter()
        .zip(sma2.iter())
        .zip(sma3.iter())
        .zip(sma4.iter())
        .map(|(((a, b), c), d)| match (a, b, c, d) {
            (a, b, c, d) if !a.is_nan() && !b.is_nan() && !c.is_nan() && !d.is_nan() => {
                *a + 2.0 * *b + 3.0 * *c + 4.0 * *d
            }
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_kst_store(store: &CandleStore) -> Option<f64> {
    // Need at least 30 + 15 = 45 bars for the last value
    if store.len() < 45 {
        return None;
    }
    let roc_at = |index: usize, period: usize| -> f64 {
        if index < period || store.close[index - period] == 0.0 {
            f64::NAN
        } else {
            100.0 * (store.close[index] - store.close[index - period]) / store.close[index - period]
        }
    };
    let sma_roc = |roc_period: usize, sma_period: usize| -> Option<f64> {
        let end = store.len() - 1;
        let start = end + 1 - sma_period;
        let mut sum = 0.0;
        for i in start..=end {
            let v = roc_at(i, roc_period);
            if v.is_nan() {
                return None;
            }
            sum += v;
        }
        Some(sum / sma_period as f64)
    };
    let s1 = sma_roc(10, 10)?;
    let s2 = sma_roc(15, 10)?;
    let s3 = sma_roc(20, 10)?;
    let s4 = sma_roc(30, 15)?;
    Some(s1 + 2.0 * s2 + 3.0 * s3 + 4.0 * s4)
}
pub fn sma_from_series(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    for index in period - 1..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        out[index] = window.iter().sum::<f64>() / period as f64;
    }
    out
}

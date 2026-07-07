use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Relative Volatility Index: RSI applied to standard deviation instead of price.
/// Uses a 10-bar stddev, then applies RSI(14) logic to up/down stddev days.
pub fn relative_volatility_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("rvi_vol:close:{period}");
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    let stddev_period = 10usize;
    if len < stddev_period + period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // Compute rolling stddev
    let mut sd = vec![f64::NAN; len];
    for i in stddev_period - 1..len {
        let window = &store.close[i + 1 - stddev_period..=i];
        let mean = window.iter().sum::<f64>() / stddev_period as f64;
        let var = window.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / stddev_period as f64;
        sd[i] = var.sqrt();
    }
    // Apply RSI logic to stddev: up_sd when close > prev_close, down_sd otherwise
    let mut up_avg = 0.0f64;
    let mut down_avg = 0.0f64;
    let mut count = 0usize;
    for i in stddev_period..len {
        if sd[i].is_nan() {
            continue;
        }
        let is_up = store.close[i] > store.close[i - 1];
        let up_val = if is_up { sd[i] } else { 0.0 };
        let down_val = if !is_up { sd[i] } else { 0.0 };
        count += 1;
        if count <= period {
            up_avg += up_val;
            down_avg += down_val;
            if count == period {
                up_avg /= period as f64;
                down_avg /= period as f64;
                let total = up_avg + down_avg;
                out[i] = if total > 0.0 {
                    (up_avg / total) * 100.0
                } else {
                    50.0
                };
            }
        } else {
            up_avg = (up_avg * (period as f64 - 1.0) + up_val) / period as f64;
            down_avg = (down_avg * (period as f64 - 1.0) + down_val) / period as f64;
            let total = up_avg + down_avg;
            out[i] = if total > 0.0 {
                (up_avg / total) * 100.0
            } else {
                50.0
            };
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_relative_volatility_store(store: &CandleStore, period: usize) -> Option<f64> {
    relative_volatility_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

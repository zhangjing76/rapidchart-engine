use crate::NodeCache;
use crate::{CandleStore};
use std::collections::HashMap;

/// Relative Vigor Index (RVI):
/// Numerator = (close-open) + 2*(close[1]-open[1]) + 2*(close[2]-open[2]) + (close[3]-open[3]) / 6
/// Denominator = (high-low) + 2*(high[1]-low[1]) + 2*(high[2]-low[2]) + (high[3]-low[3]) / 6
/// RVI = SMA(numerator/denominator, period)
/// Signal = (RVI + 2*RVI[1] + 2*RVI[2] + RVI[3]) / 6
pub fn relative_vigor_store(store: &CandleStore, period: usize, _nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    let len = store.len();
    let mut rvi_out = vec![f64::NAN; len];
    let mut signal_out = vec![f64::NAN; len];
    if period == 0 || len < period + 3 { 
        return vec![
            crate::named_series("value", rvi_out),
            crate::named_series("signal", signal_out),
        ];
    }
    // Compute smoothed numerator and denominator
    let mut num = vec![f64::NAN; len];
    let mut den = vec![f64::NAN; len];
    for i in 3..len {
        let n = (store.close[i] - store.open[i])
            + 2.0 * (store.close[i-1] - store.open[i-1])
            + 2.0 * (store.close[i-2] - store.open[i-2])
            + (store.close[i-3] - store.open[i-3]);
        let d = (store.high[i] - store.low[i])
            + 2.0 * (store.high[i-1] - store.low[i-1])
            + 2.0 * (store.high[i-2] - store.low[i-2])
            + (store.high[i-3] - store.low[i-3]);
        num[i] = n / 6.0;
        den[i] = d / 6.0;
    }
    // SMA of num/den ratio over period
    for i in (period + 2)..len {
        let mut sum_n = 0.0;
        let mut sum_d = 0.0;
        for j in (i + 1 - period)..=i {
            if !num[j].is_nan() { sum_n += num[j]; }
            if !den[j].is_nan() { sum_d += den[j]; }
        }
        if sum_d.abs() > 1e-10 {
            rvi_out[i] = sum_n / sum_d;
        }
    }
    // Signal line: symmetric weighted average
    for i in 3..len {
        if !rvi_out[i].is_nan() && !rvi_out[i-1].is_nan()
            && !rvi_out[i-2].is_nan() && !rvi_out[i-3].is_nan() {
            signal_out[i] = (rvi_out[i] + 2.0*rvi_out[i-1] + 2.0*rvi_out[i-2] + rvi_out[i-3]) / 6.0;
        }
    }
    vec![
        crate::named_series("value", rvi_out),
        crate::named_series("signal", signal_out),
    ]
}


pub fn latest_relative_vigor_store(store: &CandleStore, period: usize) -> (Option<f64>, Option<f64>) {
    let outputs = relative_vigor_store(store, period, &mut HashMap::new());
    let v = outputs[0].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    let s = outputs[1].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    (v, s)
}
use crate::NodeCache;
use crate::{Bar, CandleStore, IndicatorOutput};
use crate::indicators::sma::{sma_close, sma_close_store};
use crate::series::rc_into_owned;

/// Moving Average Cross: Two SMAs (fast and slow) with a difference histogram.
/// Outputs: fast, slow, histogram (fast - slow).
pub fn ma_cross(
    bars: &[Bar],
    fast_period: usize,
    slow_period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let fast = sma_close(bars, fast_period, nodes);
    let slow = sma_close(bars, slow_period, nodes);
    let histogram: Vec<f64> = fast
        .iter()
        .zip(slow.iter())
        .map(|(f, s)| {
            if f.is_nan() || s.is_nan() {
                f64::NAN
            } else {
                f - s
            }
        })
        .collect();
    vec![
        IndicatorOutput { name: "fast".to_string(), values: fast },
        IndicatorOutput { name: "slow".to_string(), values: slow },
        IndicatorOutput { name: "histogram".to_string(), values: histogram },
    ]
}

pub fn ma_cross_store(
    store: &CandleStore,
    fast_period: usize,
    slow_period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let fast = rc_into_owned(sma_close_store(store, fast_period, nodes));
    let slow = rc_into_owned(sma_close_store(store, slow_period, nodes));
    let histogram: Vec<f64> = fast
        .iter()
        .zip(slow.iter())
        .map(|(f, s)| {
            if f.is_nan() || s.is_nan() {
                f64::NAN
            } else {
                f - s
            }
        })
        .collect();
    vec![
        IndicatorOutput { name: "fast".to_string(), values: fast },
        IndicatorOutput { name: "slow".to_string(), values: slow },
        IndicatorOutput { name: "histogram".to_string(), values: histogram },
    ]
}

pub fn latest_ma_cross_store(
    store: &CandleStore,
    fast_period: usize,
    slow_period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let fast = crate::indicators::sma::latest_sma_store(store, fast_period);
    let slow = crate::indicators::sma::latest_sma_store(store, slow_period);
    let histogram = match (fast, slow) {
        (Some(f), Some(s)) => Some(f - s),
        _ => None,
    };
    (fast, slow, histogram)
}

use crate::NodeCache;
use crate::{Bar, CandleStore, IndicatorOutput, Series};

/// Simple Moving Average over a slice of f64 values.
fn sma_series(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    let mut sum: f64 = values[..period].iter().sum();
    out[period - 1] = sum / period as f64;
    for i in period..values.len() {
        sum += values[i] - values[i - period];
        out[i] = sum / period as f64;
    }
    out
}

/// High Low Bands: SMA of highs (upper), SMA of lows (lower), midpoint of upper+lower (middle).
pub fn high_low_bands(
    bars: &[Bar],
    period: usize,
    _nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let highs: Vec<f64> = bars.iter().map(|b| b.high).collect();
    let lows: Vec<f64> = bars.iter().map(|b| b.low).collect();
    let upper = sma_series(&highs, period);
    let lower = sma_series(&lows, period);
    let middle: Vec<f64> = upper
        .iter()
        .zip(lower.iter())
        .map(|(u, l)| {
            if u.is_nan() || l.is_nan() {
                f64::NAN
            } else {
                (u + l) / 2.0
            }
        })
        .collect();
    vec![
        IndicatorOutput { name: "upper".to_string(), values: upper },
        IndicatorOutput { name: "middle".to_string(), values: middle },
        IndicatorOutput { name: "lower".to_string(), values: lower },
    ]
}

pub fn high_low_bands_store(
    store: &CandleStore,
    period: usize,
    _nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let upper = sma_series(&store.high, period);
    let lower = sma_series(&store.low, period);
    let middle: Vec<f64> = upper
        .iter()
        .zip(lower.iter())
        .map(|(u, l)| {
            if u.is_nan() || l.is_nan() {
                f64::NAN
            } else {
                (u + l) / 2.0
            }
        })
        .collect();
    vec![
        IndicatorOutput { name: "upper".to_string(), values: upper },
        IndicatorOutput { name: "middle".to_string(), values: middle },
        IndicatorOutput { name: "lower".to_string(), values: lower },
    ]
}

pub fn latest_high_low_bands_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let start = store.len() - period;
    let upper: f64 = store.high[start..].iter().sum::<f64>() / period as f64;
    let lower: f64 = store.low[start..].iter().sum::<f64>() / period as f64;
    let middle = (upper + lower) / 2.0;
    (Some(upper), Some(middle), Some(lower))
}

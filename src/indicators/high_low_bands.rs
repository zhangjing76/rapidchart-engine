use crate::NodeCache;
use crate::{CandleStore, Series};

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

/// High Low Bands: SMA of highs (upper) of lows (lower), midpoint of upper+lower (middle).

pub fn high_low_bands_store(
    store: &CandleStore,
    period: usize,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
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
        crate::named_series("upper", upper),
        crate::named_series("middle", middle),
        crate::named_series("lower", lower),
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
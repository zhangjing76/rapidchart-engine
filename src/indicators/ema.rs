use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn ema_close_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ema:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(ema_values(store.close.iter().copied(), period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn ema_values(values: impl IntoIterator<Item = f64>, period: usize) -> Series {
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut current = None::<f64>;
    let mut out = Vec::new();
    for value in values {
        let next = match current {
            Some(previous) => alpha * value + (1.0 - alpha) * previous,
            None => value,
        };
        current = Some(next);
        out.push(next);
    }
    out
}
pub fn ema_series(values: &[f64], period: usize) -> Series {
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut current = None::<f64>;
    let mut out = Vec::with_capacity(values.len());
    for &value in values {
        if value.is_nan() {
            out.push(f64::NAN);
        } else {
            let next = match current {
                Some(previous) => alpha * value + (1.0 - alpha) * previous,
                None => value,
            };
            current = Some(next);
            out.push(next);
        }
    }
    out
}
pub fn latest_ema_store(store: &CandleStore, period: usize, output: Option<&[f64]>) -> Option<f64> {
    let last = store.last_close()?;
    if period == 0 || store.len() == 1 {
        return Some(last);
    }
    let previous = output
        .and_then(|values| values.get(store.len() - 2))
        .copied()
        .and_then(nan_to_none)
        .unwrap_or(store.close[store.len() - 2]);
    Some(ema_next(last, previous, period))
}
pub fn ema_next(value: f64, previous: f64, period: usize) -> f64 {
    let alpha = 2.0 / (period as f64 + 1.0);
    alpha * value + (1.0 - alpha) * previous
}

/// Exponential moving average with the usual start: no value until `period` values have been
/// seen, then the simple average of those values, then the recursive average. NaN inputs are
/// skipped and stay NaN, so an average of another average starts once that one has values.
pub fn ema_sma_seeded(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 {
        return out;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut seen = 0usize;
    let mut sum = 0.0;
    let mut current = f64::NAN;
    for (index, &value) in values.iter().enumerate() {
        if value.is_nan() {
            continue;
        }
        seen += 1;
        if seen < period {
            sum += value;
            continue;
        }
        current = if seen == period {
            (sum + value) / period as f64
        } else {
            alpha * value + (1.0 - alpha) * current
        };
        out[index] = current;
    }
    out
}

/// [`ema_sma_seeded`] of the close, cached by period.
pub fn ema_sma_seeded_close_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("ema_sma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(ema_sma_seeded(&store.close, period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

/// Latest [`ema_sma_seeded`] of the close, from the previous bar's value when there is one.
pub fn latest_ema_sma_seeded_store(
    store: &CandleStore,
    period: usize,
    output: Option<&[f64]>,
) -> Option<f64> {
    let last = store.last_close()?;
    if period == 0 || store.len() < period {
        return None;
    }
    let previous = store
        .len()
        .checked_sub(2)
        .and_then(|index| output.and_then(|values| values.get(index)).copied())
        .and_then(nan_to_none);
    match previous {
        Some(previous) => Some(ema_next(last, previous, period)),
        // The bar where the average starts, or no earlier output: calculate it from the closes.
        None => ema_sma_seeded(&store.close, period)
            .last()
            .copied()
            .and_then(nan_to_none),
    }
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("EMA", "EMA", "Moving Average", "overlay", 50)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn close_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            vec![1.0; len],
        )
    }

    fn assert_series_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else {
                assert!((actual - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn ema_is_the_manual_recursive_average() {
        let store = close_store(&[10.0, 12.0, 14.0, 13.0]);
        let values = ema_close_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[10.0, 11.0, 12.5, 12.75]);
        assert_eq!(latest_ema_store(&store, 3, Some(&values[..])), Some(12.75));
    }

    #[test]
    fn seeded_ema_starts_with_the_simple_average() {
        let nan = f64::NAN;
        // Period 3: nothing for two bars, then (10 + 12 + 14) / 3, then the usual recursion.
        let values = ema_sma_seeded(&[10.0, 12.0, 14.0, 13.0, 15.0], 3);
        assert_series_close(&values, &[nan, nan, 12.0, 12.5, 13.75]);
    }

    #[test]
    fn seeded_ema_skips_leading_gaps() {
        let nan = f64::NAN;
        // An average of another average: the first two inputs have no value yet.
        let values = ema_sma_seeded(&[nan, nan, 10.0, 12.0, 14.0], 2);
        assert_series_close(&values, &[nan, nan, nan, 11.0, 13.0]);
        assert_series_close(&ema_sma_seeded(&[1.0, 2.0], 0), &[nan, nan]);
    }

    #[test]
    fn latest_seeded_ema_matches_the_full_calculation() {
        let closes: Vec<f64> = (0..40).map(|i| 50.0 + ((i * 7) % 11) as f64).collect();
        for period in [1, 2, 3, 12, 26, 40] {
            let full = ema_sma_seeded(&closes, period);
            for len in period..=closes.len() {
                let store = close_store(&closes[..len]);
                let previous = &full[..len - 1];
                let latest = latest_ema_sma_seeded_store(&store, period, Some(previous)).unwrap();
                assert!(
                    (latest - full[len - 1]).abs() < 1e-9,
                    "period {period}, len {len}"
                );
            }
        }
        assert_eq!(
            latest_ema_sma_seeded_store(&close_store(&[1.0, 2.0]), 3, None),
            None
        );
        let store = close_store(&[10.0, 12.0, 14.0]);
        assert_eq!(latest_ema_sma_seeded_store(&store, 3, None), Some(12.0));
    }
}

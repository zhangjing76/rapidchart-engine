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

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
                "RELATIVE_VOLATILITY",
                "RELATIVE VOLATILITY",
                "Volatility",
                "separate",
                14,
            )
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
    fn relative_volatility_is_fifty_for_flat_prices() {
        let store = close_store(&[1.0; 12]);
        let values = relative_volatility_store(&store, 2, &mut HashMap::new());

        assert_series_close(
            &values,
            &[
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                50.0,
            ],
        );
        assert_eq!(latest_relative_volatility_store(&store, 2), Some(50.0));
    }

    #[test]
    fn relative_volatility_is_hundred_when_all_stddev_days_are_up() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);
        let values = relative_volatility_store(&store, 2, &mut HashMap::new());

        assert_series_close(
            &values,
            &[
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                100.0,
            ],
        );
        assert_eq!(latest_relative_volatility_store(&store, 2), Some(100.0));
    }
}

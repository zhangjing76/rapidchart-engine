use crate::indicators::kst::sma_from_series;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Ulcer Index (Peter Martin): the root of the average squared percentage drawdown.
/// Each bar's drawdown is R = 100 * (close / highest close of the last `period` bars - 1), and
/// UI = sqrt(SMA(R^2, period)). The first drawdown is at bar period - 1 and the first value at
/// bar 2 * (period - 1).
pub fn ulcer_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ulcer_dd:close:{period}");
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut drawdown_squared = vec![f64::NAN; len];
    if period > 0 {
        for i in period - 1..len {
            let highest = store.close[i + 1 - period..=i]
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            if highest.abs() >= 1e-10 {
                let drawdown = 100.0 * (store.close[i] / highest - 1.0);
                drawdown_squared[i] = drawdown * drawdown;
            }
        }
    }
    let out: Vec<f64> = sma_from_series(&drawdown_squared, period)
        .iter()
        .map(|mean| mean.sqrt())
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_ulcer_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    ulcer_index_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "ULCER_INDEX",
        "ULCER INDEX",
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
    fn ulcer_index_is_zero_when_prices_do_not_fall() {
        let store = close_store(&[1.0, 1.0, 1.0, 1.0]);
        let values = ulcer_index_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 0.0, 0.0]);
        assert_eq!(latest_ulcer_index_store(&store, 2), Some(0.0));
    }

    #[test]
    fn ulcer_index_averages_each_bars_own_drawdown() {
        let store = close_store(&[100.0, 50.0, 100.0, 80.0]);
        let values = ulcer_index_store(&store, 2, &mut HashMap::new());

        // Drawdowns over two-bar windows from bar 1: -50, 0, -20. The first value is at bar 2
        // and averages the squares of the drawdowns at bars 1 and 2: sqrt((2500 + 0) / 2).
        assert_series_close(
            &values,
            &[
                f64::NAN,
                f64::NAN,
                (2500.0_f64 / 2.0).sqrt(),
                (400.0_f64 / 2.0).sqrt(),
            ],
        );
    }
}

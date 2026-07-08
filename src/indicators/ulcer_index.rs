use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Ulcer Index: measures downside volatility/drawdown severity.
/// UI = sqrt(SUM(((close - max_close_over_period) / max_close_over_period * 100)^2, period) / period)
pub fn ulcer_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ulcer:close:{period}");
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..len {
        let max_close = store.close[i + 1 - period..=i]
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        if max_close.abs() < 1e-10 {
            continue;
        }
        let sum_sq: f64 = store.close[i + 1 - period..=i]
            .iter()
            .map(|&c| {
                let pct_drawdown = ((c - max_close) / max_close) * 100.0;
                pct_drawdown * pct_drawdown
            })
            .sum();
        out[i] = (sum_sq / period as f64).sqrt();
    }
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
    fn ulcer_index_is_zero_when_prices_make_new_highs() {
        let store = close_store(&[1.0, 1.0, 1.0]);
        let values = ulcer_index_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 0.0]);
        assert_eq!(latest_ulcer_index_store(&store, 3), Some(0.0));
    }
}

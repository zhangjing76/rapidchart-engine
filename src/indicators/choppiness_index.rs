use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Choppiness Index:
/// CI = 100 * LOG10(SUM(ATR(1), period) / (Highest_High - Lowest_Low)) / LOG10(period)
/// Range: 0-100. High values (>61.8) = choppy/sideways, Low values (<38.2) = trending.
pub fn choppiness_index_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("chop:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period + 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // True Range series
    let mut tr = vec![0.0f64; len];
    for i in 1..len {
        tr[i] = (store.high[i] - store.low[i])
            .max((store.high[i] - store.close[i - 1]).abs())
            .max((store.low[i] - store.close[i - 1]).abs());
    }
    let log_period = (period as f64).log10();
    for i in period..len {
        let sum_atr: f64 = tr[i + 1 - period..=i].iter().sum();
        let hh = store.high[i + 1 - period..=i]
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = store.low[i + 1 - period..=i]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let range = hh - ll;
        if range > 1e-10 && log_period > 0.0 {
            out[i] = 100.0 * (sum_atr / range).log10() / log_period;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_choppiness_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    choppiness_index_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
                "CHOPPINESS_INDEX",
                "CHOPPINESS INDEX",
                "Volatility",
                "separate",
                14,
            )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[f64]) -> CandleStore {
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
    fn choppiness_index_matches_the_manual_formula() {
        let store = ohlc_store(&[1.0, 2.0, 3.0, 4.0]);
        let values = choppiness_index_store(&store, 3, &mut HashMap::new());

        let expected = 100.0 * (3.0f64 / 2.0f64).log10() / (3.0f64).log10();
        assert_series_close(&values, &[f64::NAN, f64::NAN, f64::NAN, expected]);
        assert_eq!(latest_choppiness_index_store(&store, 3), Some(expected));
    }
}

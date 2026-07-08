use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Gopalakrishnan Range Index: log(highest - lowest over period) / log(period)
pub fn gopalakrishnan_range_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("gapo:hl:{period}");
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let log_period = (period as f64).ln();
    for i in period - 1..len {
        let hh = store.high[i + 1 - period..=i]
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = store.low[i + 1 - period..=i]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let range = hh - ll;
        if range > 0.0 && log_period > 0.0 {
            out[i] = range.ln() / log_period;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_gopalakrishnan_range_store(store: &CandleStore, period: usize) -> Option<f64> {
    gopalakrishnan_range_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "GOPALAKRISHNAN_RANGE",
        "GOPALAKRISHNAN RANGE INDEX",
        "Volatility",
        "separate",
        14,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
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
    fn gopalakrishnan_range_matches_log_scaled_range() {
        let store = ohlc_store(&[(4.0, 1.0, 2.0), (6.0, 2.0, 4.0), (8.0, 3.0, 6.0)]);
        let values = gopalakrishnan_range_store(&store, 3, &mut HashMap::new());
        let expected = (7.0_f64).ln() / (3.0_f64).ln();

        assert_series_close(&values, &[f64::NAN, f64::NAN, expected]);
        assert_eq!(latest_gopalakrishnan_range_store(&store, 3), Some(expected));
    }
}

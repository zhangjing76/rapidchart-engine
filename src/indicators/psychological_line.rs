use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Psychological Line: percentage of bars that closed up over the period.
/// PSY = (bars where close > prev_close) / period * 100
pub fn psychological_line_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("psy:close:{period}");
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period + 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period..len {
        let up = (i + 1 - period..=i)
            .filter(|&j| store.close[j] > store.close[j - 1])
            .count();
        out[i] = (up as f64 / period as f64) * 100.0;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_psychological_line_store(store: &CandleStore, period: usize) -> Option<f64> {
    psychological_line_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
                "PSYCHOLOGICAL_LINE",
                "PSYCHOLOGICAL LINE",
                "Trend Analysis",
                "separate",
                12,
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
    fn psychological_line_counts_up_closes() {
        let store = close_store(&[1.0, 2.0, 1.0, 3.0]);
        let values = psychological_line_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 50.0, 50.0]);
        assert_eq!(latest_psychological_line_store(&store, 2), Some(50.0));
    }

    #[test]
    fn psychological_line_reaches_hundred_when_every_bar_is_up() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0]);
        let values = psychological_line_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 100.0, 100.0]);
        assert_eq!(latest_psychological_line_store(&store, 2), Some(100.0));
    }
}

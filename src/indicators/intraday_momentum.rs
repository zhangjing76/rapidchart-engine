use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Intraday Momentum Index (IMI):
/// IMI = (sum of gains / (sum of gains + sum of losses)) * 100
/// where gain = close - open when close > open, loss = open - close when close < open
/// Computed over a rolling window of `period` bars.
pub fn intraday_momentum_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("imi:oc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..len {
        let mut gains = 0.0;
        let mut losses = 0.0;
        for j in i + 1 - period..=i {
            let diff = store.close[j] - store.open[j];
            if diff > 0.0 {
                gains += diff;
            } else {
                losses += -diff;
            }
        }
        let total = gains + losses;
        out[i] = if total > 0.0 {
            (gains / total) * 100.0
        } else {
            50.0
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_intraday_momentum_store(store: &CandleStore, period: usize) -> Option<f64> {
    intraday_momentum_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[(f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, close)| *close).collect(),
            values.iter().map(|(_, close)| *close).collect(),
            values.iter().map(|(_, close)| *close).collect(),
            values.iter().map(|(_, close)| *close).collect(),
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
    fn intraday_momentum_is_fifty_when_all_bars_are_flat() {
        let store = ohlc_store(&[(2.0, 2.0), (2.0, 2.0), (2.0, 2.0)]);
        let values = intraday_momentum_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, 50.0, 50.0]);
        assert_eq!(latest_intraday_momentum_store(&store, 2), Some(50.0));
    }
}

use crate::indicators::donchian::donchian_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

const UPPER_SLOT: usize = 0;
const LOWER_SLOT: usize = 2;

/// Donchian Width: the Donchian Channel's upper minus lower line, in price points. It uses the
/// same window as the channel (the previous `period` bars, not counting the current bar), so the
/// first value is at bar `period`.
pub fn donchian_width_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("donchian_width_points:{}", period);
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let dc = donchian_store(store, period, nodes);
    let upper = &dc[UPPER_SLOT].values;
    let lower = &dc[LOWER_SLOT].values;
    let out: Vec<f64> = upper.iter().zip(lower.iter()).map(|(u, l)| u - l).collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_donchian_width_store(store: &CandleStore, period: usize) -> Option<f64> {
    donchian_width_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "DONCHIAN_WIDTH",
        "DONCHIAN WIDTH",
        "Volatility",
        "separate",
        20,
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
    fn donchian_width_is_zero_for_flat_range() {
        let store = ohlc_store(&[(2.0, 2.0, 2.0), (2.0, 2.0, 2.0), (2.0, 2.0, 2.0)]);
        let values = donchian_width_store(&store, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 0.0]);
        assert_eq!(latest_donchian_width_store(&store, 2), Some(0.0));
    }

    #[test]
    fn donchian_width_spans_the_previous_period_bars() {
        let store = ohlc_store(&[
            (2.0, 0.0, 1.0),
            (4.0, 2.0, 3.0),
            (6.0, 4.0, 5.0),
            (9.0, 1.0, 5.0),
        ]);
        let values = donchian_width_store(&store, 2, &mut HashMap::new());

        // Bar 2: bars 0-1 give 4 - 0. Bar 3: bars 1-2 give 6 - 2; bar 3's own range is ignored.
        assert_series_close(&values, &[f64::NAN, f64::NAN, 4.0, 4.0]);
        assert_eq!(latest_donchian_width_store(&store, 2), Some(4.0));
    }
}

use crate::indicators::donchian::donchian_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Donchian Width: (upper - lower) / middle * 100
const UPPER_SLOT: usize = 0;
const MIDDLE_SLOT: usize = 1;
const LOWER_SLOT: usize = 2;

pub fn donchian_width_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("donchian_width:{}", period);
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let dc = donchian_store(store, period, nodes);
    let upper = &dc[UPPER_SLOT].values;
    let middle = &dc[MIDDLE_SLOT].values;
    let lower = &dc[LOWER_SLOT].values;
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        if !upper[i].is_nan()
            && !middle[i].is_nan()
            && !lower[i].is_nan()
            && middle[i].abs() > 1e-10
        {
            out[i] = ((upper[i] - lower[i]) / middle[i]) * 100.0;
        }
    }
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

        assert_series_close(&values, &[f64::NAN, 0.0, 0.0]);
        assert_eq!(latest_donchian_width_store(&store, 2), Some(0.0));
    }
}

use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Vertical Horizontal Filter:
/// VHF = |close - close[period]| / SUM(|close[i] - close[i-1]|, period)
/// High VHF = trending, Low VHF = ranging.
pub fn vertical_horizontal_filter_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("vhf:close:{period}");
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
        let numerator = (store.close[i] - store.close[i - period]).abs();
        let denominator: f64 = (i + 1 - period..=i)
            .map(|j| (store.close[j] - store.close[j - 1]).abs())
            .sum();
        if denominator > 1e-10 {
            out[i] = numerator / denominator;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_vertical_horizontal_filter_store(store: &CandleStore, period: usize) -> Option<f64> {
    vertical_horizontal_filter_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
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

    #[test]
    fn vertical_horizontal_filter_is_one_for_a_perfect_trend() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0]);
        let values = vertical_horizontal_filter_store(&store, 2, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!((values[2] - 1.0).abs() < 1e-12);
        assert!((values[3] - 1.0).abs() < 1e-12);
        assert_eq!(latest_vertical_horizontal_filter_store(&store, 2), Some(1.0));
    }
}

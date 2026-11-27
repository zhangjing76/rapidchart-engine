use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Vertical Horizontal Filter (Adam White):
/// VHF = (highest close - lowest close) / SUM(|close[i] - close[i-1]|, period), with the range
/// taken over the period + 1 closes those changes span, so a straight line gives 1.
/// High VHF = trending, Low VHF = ranging. The first value is at bar `period`.
pub fn vertical_horizontal_filter_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("vhf_range:close:{period}");
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
        let window = &store.close[i - period..=i];
        let highest = window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = window.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let denominator: f64 = (i + 1 - period..=i)
            .map(|j| (store.close[j] - store.close[j - 1]).abs())
            .sum();
        if denominator > 1e-10 {
            out[i] = (highest - lowest) / denominator;
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

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "VERTICAL_HORIZONTAL_FILTER",
        "VERTICAL HORIZONTAL FILTER",
        "Trend Analysis",
        "separate",
        28,
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

    #[test]
    fn vertical_horizontal_filter_is_one_for_a_perfect_trend() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0]);
        let values = vertical_horizontal_filter_store(&store, 2, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!((values[2] - 1.0).abs() < 1e-12);
        assert!((values[3] - 1.0).abs() < 1e-12);
        assert_eq!(
            latest_vertical_horizontal_filter_store(&store, 2),
            Some(1.0)
        );
    }

    #[test]
    fn vertical_horizontal_filter_is_low_when_price_goes_back_and_forth() {
        let store = close_store(&[1.0, 3.0, 1.0, 3.0, 1.0]);
        let values = vertical_horizontal_filter_store(&store, 4, &mut HashMap::new());

        // The range is 2 but the closes travel 8.
        assert!((values[4] - 0.25).abs() < 1e-12);
    }
}

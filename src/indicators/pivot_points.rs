use crate::CandleStore;
use crate::NodeCache;
use std::rc::Rc;

type PivotPointsResult = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

pub fn pivot_points_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    let mut pp = vec![f64::NAN; store.len()];
    let mut r1 = vec![f64::NAN; store.len()];
    let mut s1 = vec![f64::NAN; store.len()];
    let mut r2 = vec![f64::NAN; store.len()];
    let mut s2 = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        let pivot = (store.high[index - 1] + store.low[index - 1] + store.close[index - 1]) / 3.0;
        let range = store.high[index - 1] - store.low[index - 1];
        pp[index] = pivot;
        r1[index] = 2.0 * pivot - store.low[index - 1];
        s1[index] = 2.0 * pivot - store.high[index - 1];
        r2[index] = pivot + range;
        s2[index] = pivot - range;
    }
    nodes.insert("pivot:pp".to_string(), Rc::new(pp.clone()));
    nodes.insert("pivot:r1".to_string(), Rc::new(r1.clone()));
    nodes.insert("pivot:s1".to_string(), Rc::new(s1.clone()));
    nodes.insert("pivot:r2".to_string(), Rc::new(r2.clone()));
    nodes.insert("pivot:s2".to_string(), Rc::new(s2.clone()));
    vec![
        crate::named_series("pp", pp),
        crate::named_series("r1", r1),
        crate::named_series("s1", s1),
        crate::named_series("r2", r2),
        crate::named_series("s2", s2),
    ]
}
pub fn latest_pivot_points_store(store: &CandleStore) -> PivotPointsResult {
    if store.len() < 2 {
        return (None, None, None, None, None);
    }
    let index = store.len() - 2;
    let pivot = (store.high[index] + store.low[index] + store.close[index]) / 3.0;
    let range = store.high[index] - store.low[index];
    (
        Some(pivot),
        Some(2.0 * pivot - store.low[index]),
        Some(2.0 * pivot - store.high[index]),
        Some(pivot + range),
        Some(pivot - range),
    )
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "PIVOT_POINTS",
                name: "PIVOT POINTS",
                category: "Support/Resistance",
                pane: "overlay",
                params: Vec::new(),
                outputs: vec![
                    crate::descriptors::output_descriptor("pp", "line", "overlay", "#64748b"),
                    crate::descriptors::output_descriptor("r1", "line", "overlay", "#059669"),
                    crate::descriptors::output_descriptor("s1", "line", "overlay", "#dc2626"),
                    crate::descriptors::output_descriptor("r2", "line", "overlay", "#16a34a"),
                    crate::descriptors::output_descriptor("s2", "line", "overlay", "#b91c1c"),
                ],
            }
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
    fn pivot_points_use_the_previous_bar() {
        let store = ohlc_store(&[(5.0, 1.0, 3.0), (8.0, 4.0, 6.0)]);
        let values = pivot_points_store(&store, &mut HashMap::new());

        assert_series_close(&values[0].values, &[f64::NAN, 3.0]);
        assert_series_close(&values[1].values, &[f64::NAN, 5.0]);
        assert_series_close(&values[2].values, &[f64::NAN, 1.0]);
        assert_series_close(&values[3].values, &[f64::NAN, 7.0]);
        assert_series_close(&values[4].values, &[f64::NAN, -1.0]);
        assert_eq!(
            latest_pivot_points_store(&store),
            (Some(3.0), Some(5.0), Some(1.0), Some(7.0), Some(-1.0))
        );
    }
}

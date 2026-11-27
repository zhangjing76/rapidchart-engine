use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Wilder's Swing Index for bar `i` (which must be at least 1) with limit move `limit`:
/// SI = 50 * ((C - Cy) + 0.5 * (C - O) + 0.25 * (Cy - Oy)) / R * K / T, where
/// A = |H - Cy|, B = |L - Cy|, C' = H - L, D = |Cy - Oy|, K = max(A, B) and R is
/// A - 0.5B + 0.25D, B - 0.5A + 0.25D or C' + 0.25D for whichever of A, B, C' is largest.
fn swing_at(store: &CandleStore, i: usize, limit: f64) -> f64 {
    let (open, high, low, close) = (store.open[i], store.high[i], store.low[i], store.close[i]);
    let (prev_open, prev_close) = (store.open[i - 1], store.close[i - 1]);
    let a = (high - prev_close).abs();
    let b = (low - prev_close).abs();
    let range = (high - low).abs();
    let d = (prev_close - prev_open).abs();
    let k = a.max(b);
    let largest = range.max(k);
    let r = if largest == a {
        a - 0.5 * b + 0.25 * d
    } else if largest == b {
        b - 0.5 * a + 0.25 * d
    } else {
        range + 0.25 * d
    };
    if r == 0.0 {
        return 0.0;
    }
    let change = (close - prev_close) + 0.5 * (close - open) + 0.25 * (prev_close - prev_open);
    50.0 * change / r * (k / limit)
}

/// Swing Index (Wilder), one value per bar from bar 1. `limit` is the limit move value T, which
/// must be greater than zero.
pub fn swing_index_store(store: &CandleStore, limit: f64, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("swing_index_wilder:ohlc:{limit}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    for (i, value) in out.iter_mut().enumerate().skip(1) {
        *value = swing_at(store, i, limit);
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_swing_index_store(store: &CandleStore, limit: f64) -> Option<f64> {
    if store.len() < 2 {
        return None;
    }
    crate::nan_to_none(swing_at(store, store.len() - 1, limit))
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "SWING_INDEX",
        name: "SWING INDEX",
        category: "Trend Analysis",
        pane: "separate",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "multiplier",
            label: "Limit Move Value",
            default: 0.5,
            min: 0.01,
            step: "0.01",
        }],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#2563eb",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[(f64, f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(high, _, _, _)| *high).collect(),
            values.iter().map(|(_, low, _, _)| *low).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(_, _, _, volume)| *volume).collect(),
        )
    }

    #[test]
    fn swing_index_is_zero_for_flat_prices() {
        let store = ohlc_store(&[
            (10.0, 10.0, 10.0, 1.0),
            (10.0, 10.0, 10.0, 1.0),
            (10.0, 10.0, 10.0, 1.0),
        ]);
        let values = swing_index_store(&store, 0.5, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert_eq!(values[1], 0.0);
        assert_eq!(values[2], 0.0);
        assert_eq!(latest_swing_index_store(&store, 0.5), Some(0.0));
    }

    #[test]
    fn swing_index_follows_wilders_formula_on_each_bar() {
        // Open, high, low, close per bar.
        let store = CandleStore::from_raw_columns(
            vec![0, 1, 2],
            vec![10.0, 11.0, 10.0],
            vec![12.0, 13.0, 14.0],
            vec![9.0, 10.0, 11.0],
            vec![11.0, 12.0, 13.0],
            vec![1.0, 1.0, 1.0],
        );
        let values = swing_index_store(&store, 0.5, &mut HashMap::new());

        assert!(values[0].is_nan());
        // Bar 1: the bar's range (3) is largest, so R = 3 + 0.25 * |11 - 10| = 3.25; K = 2 and
        // the change term is 1 + 0.5 * 1 + 0.25 * 1 = 1.75. Each bar stands alone.
        assert!((values[1] - 50.0 * 1.75 / 3.25 * (2.0 / 0.5)).abs() < 1e-9);
        // Bar 2: the range (3) is largest again, with D = |12 - 11| = 1, K = 2 and the change
        // term 1 + 0.5 * 3 + 0.25 * 1 = 2.75.
        assert!((values[2] - 50.0 * 2.75 / 3.25 * (2.0 / 0.5)).abs() < 1e-9);
        assert_eq!(latest_swing_index_store(&store, 0.5), Some(values[2]));
    }

    #[test]
    fn swing_index_uses_wilders_r_when_the_high_gap_is_largest() {
        let store = CandleStore::from_raw_columns(
            vec![0, 1],
            vec![10.0, 10.0],
            vec![10.0, 14.0],
            vec![10.0, 11.0],
            vec![10.0, 13.0],
            vec![1.0, 1.0],
        );
        let values = swing_index_store(&store, 0.5, &mut HashMap::new());

        // A = 4 is largest, so R = 4 - 0.5 * 1 = 3.5; the change term is 3 + 0.5 * 3 = 4.5.
        assert!((values[1] - 50.0 * 4.5 / 3.5 * (4.0 / 0.5)).abs() < 1e-9);
    }
}

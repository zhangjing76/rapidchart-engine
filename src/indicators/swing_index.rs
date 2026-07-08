use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Wilder's Swing Index (ASI - Accumulative Swing Index):
/// Measures the strength of price movement using OHLC relationships.
/// SI = 50 * (Cy - C + 0.5*(Cy - Oy) + 0.25*(C - O)) / R * (K / T)
/// where T is the limit move (we use ATR as proxy), R is the largest of several ranges.
pub fn swing_index_store(store: &CandleStore, _nodes: &mut NodeCache) -> RcSeries {
    let key = "swing_index:ohlc".to_string();
    if let Some(values) = _nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 {
        let rc = Rc::new(out);
        _nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let mut cumulative = 0.0;
    for i in 1..len {
        let c = store.close[i];
        let cy = store.close[i - 1];
        let o = store.open[i];
        let oy = store.open[i - 1];
        let h = store.high[i];
        let l = store.low[i];
        let _hy = store.high[i - 1];
        let _ly = store.low[i - 1];

        let k = (h - cy).abs().max((l - cy).abs());
        let tr = (h - l).max((h - cy).abs()).max((l - cy).abs());

        if tr < 1e-10 {
            out[i] = cumulative;
            continue;
        }

        let er = if (h - cy).abs() >= (l - cy).abs() && (h - cy).abs() >= (h - l) {
            (h - cy).abs() + 0.5 * (l - cy).abs() + 0.25 * (cy - oy).abs()
        } else if (l - cy).abs() >= (h - cy).abs() && (l - cy).abs() >= (h - l) {
            (l - cy).abs() + 0.5 * (h - cy).abs() + 0.25 * (cy - oy).abs()
        } else {
            (h - l) + 0.25 * (cy - oy).abs()
        };

        if er.abs() < 1e-10 {
            out[i] = cumulative;
            continue;
        }

        let si = 50.0 * ((cy - c) + 0.5 * (cy - oy) + 0.25 * (c - o)) * k / (er * tr);
        cumulative += si;
        out[i] = cumulative;
    }
    let rc = Rc::new(out);
    _nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_swing_index_store(store: &CandleStore) -> Option<f64> {
    swing_index_store(store, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "SWING_INDEX",
        name: "SWING INDEX",
        category: "Trend Analysis",
        pane: "separate",
        params: Vec::new(),
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
        let values = swing_index_store(&store, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert_eq!(values[1], 0.0);
        assert_eq!(values[2], 0.0);
        assert_eq!(latest_swing_index_store(&store), Some(0.0));
    }

    #[test]
    fn swing_index_accumulates_negative_swings_in_a_rising_open_close_sequence() {
        let store = CandleStore::from_raw_columns(
            vec![0, 1, 2],
            vec![10.0, 11.0, 12.0],
            vec![12.0, 13.0, 14.0],
            vec![9.0, 10.0, 11.0],
            vec![11.0, 12.0, 13.0],
            vec![1.0, 1.0, 1.0],
        );
        let values = swing_index_store(&store, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert!((values[1] + 2.5641025641025643).abs() < 1e-12);
        assert!((values[2] + 5.128205128205129).abs() < 1e-12);
        assert!((latest_swing_index_store(&store).unwrap() + 5.128205128205129).abs() < 1e-12);
    }
}

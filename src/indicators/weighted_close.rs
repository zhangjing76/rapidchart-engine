use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Weighted Close = (High + Low + 2*Close) / 4

pub fn weighted_close_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "weighted_close:hlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .zip(store.close.iter())
        .map(|((h, l), c)| (h + l + 2.0 * c) / 4.0)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_weighted_close_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 {
        return None;
    }
    let i = store.len() - 1;
    Some((store.high[i] + store.low[i] + 2.0 * store.close[i]) / 4.0)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "WEIGHTED_CLOSE",
                name: "WEIGHTED CLOSE",
                category: "Averages/Bands",
                pane: "overlay",
                params: Vec::new(),
                outputs: vec![crate::descriptors::output_descriptor("value", "line", "overlay", "#64748b")],
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

    #[test]
    fn weighted_close_is_the_manual_weighted_mean() {
        let store = ohlc_store(&[(12.0, 6.0, 9.0), (15.0, 9.0, 12.0)]);
        let values = weighted_close_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[9.0, 12.0]);
        assert_eq!(latest_weighted_close_store(&store), Some(12.0));
    }
}

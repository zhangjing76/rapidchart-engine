use crate::{CandleStore, NodeCache, RcSeries};
use std::rc::Rc;

/// Cached (high + low) / 2 series.
#[allow(dead_code)]
pub fn hl2_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "hl2";
    if let Some(values) = nodes.get(key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(&h, &l)| (h + l) / 2.0)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key.to_string(), Rc::clone(&rc));
    rc
}

/// Latest (high + low) / 2 value.
pub fn latest_hl2(store: &CandleStore) -> Option<f64> {
    Some((*store.high.last()? + *store.low.last()?) / 2.0)
}

/// Cached (high + low + close) / 3 series.
pub fn hlc3_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "hlc3";
    if let Some(values) = nodes.get(key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .zip(store.close.iter())
        .map(|((&h, &l), &c)| (h + l + c) / 3.0)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key.to_string(), Rc::clone(&rc));
    rc
}

/// Latest (high + low + close) / 3 value.
pub fn latest_hlc3(store: &CandleStore) -> Option<f64> {
    Some((*store.high.last()? + *store.low.last()? + *store.close.last()?) / 3.0)
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
    fn hl2_and_hlc3_are_their_manual_averages() {
        let store = ohlc_store(&[(6.0, 2.0, 4.0), (9.0, 3.0, 6.0)]);
        let mut nodes = HashMap::new();
        let hl2 = hl2_store(&store, &mut nodes);
        let hlc3 = hlc3_store(&store, &mut nodes);

        assert_eq!(&*hl2, &[4.0, 6.0]);
        assert_eq!(&*hlc3, &[4.0, 6.0]);
        assert_eq!(latest_hl2(&store), Some(6.0));
        assert_eq!(latest_hlc3(&store), Some(6.0));
    }
}

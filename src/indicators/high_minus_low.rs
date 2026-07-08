use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// High Minus Low: simple H - L for each bar.
pub fn high_minus_low_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "hml:hl".to_string();
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let out: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(h, l)| h - l)
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_high_minus_low_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 {
        return None;
    }
    let i = store.len() - 1;
    Some(store.high[i] - store.low[i])
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "HIGH_MINUS_LOW",
        name: "HIGH MINUS LOW",
        category: "Volatility",
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
    fn high_minus_low_is_the_manual_spread() {
        let store = ohlc_store(&[(4.0, 0.0, 3.0), (8.0, 2.0, 5.0)]);
        let values = high_minus_low_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[4.0, 6.0]);
        assert_eq!(latest_high_minus_low_store(&store), Some(6.0));
    }
}

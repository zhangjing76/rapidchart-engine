use crate::indicators::derived::{hlc3_store, latest_hlc3};
use crate::NodeCache;
use crate::{CandleStore, RcSeries};

/// Typical Price = (High + Low + Close) / 3 — delegates to the shared `hlc3_store` cache.
pub fn typical_price_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    hlc3_store(store, nodes)
}

pub fn latest_typical_price_store(store: &CandleStore) -> Option<f64> {
    latest_hlc3(store)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "TYPICAL_PRICE",
                name: "TYPICAL PRICE",
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
    fn typical_price_is_the_mean_of_high_low_close() {
        let store = ohlc_store(&[(12.0, 6.0, 9.0), (15.0, 9.0, 12.0)]);
        let values = typical_price_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[9.0, 12.0]);
        assert_eq!(latest_typical_price_store(&store), Some(12.0));
    }
}

use crate::indicators::derived::{hl2_store, latest_hl2};
use crate::NodeCache;
use crate::{CandleStore, RcSeries};

/// Median Price = (High + Low) / 2 — delegates to the shared `hl2_store` cache.
pub fn median_price_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    hl2_store(store, nodes)
}

pub fn latest_median_price_store(store: &CandleStore) -> Option<f64> {
    latest_hl2(store)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "MEDIAN_PRICE",
                name: "MEDIAN PRICE",
                category: "Statistical",
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
    fn median_price_is_the_midpoint_of_high_and_low() {
        let store = ohlc_store(&[(12.0, 6.0, 9.0), (15.0, 9.0, 12.0)]);
        let values = median_price_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[9.0, 12.0]);
        assert_eq!(latest_median_price_store(&store), Some(12.0));
    }
}

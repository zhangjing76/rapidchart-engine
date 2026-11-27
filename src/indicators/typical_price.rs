use crate::indicators::derived::hlc3_store;
use crate::indicators::kst::sma_from_series;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Typical Price: the `period`-bar simple average of (High + Low + Close) / 3.
pub fn typical_price_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("typical_price:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(sma_from_series(&hlc3_store(store, nodes), period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

/// Latest Typical Price, from the last `period` bars.
pub fn latest_typical_price_store(store: &CandleStore, period: usize) -> Option<f64> {
    let len = store.len();
    if period == 0 || len < period {
        return None;
    }
    let sum: f64 = (len - period..len)
        .map(|i| (store.high[i] + store.low[i] + store.close[i]) / 3.0)
        .sum();
    Some(sum / period as f64)
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "TYPICAL_PRICE",
        name: "TYPICAL PRICE",
        category: "Averages/Bands",
        pane: "overlay",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "period",
            label: "Period",
            default: 14.0,
            min: 1.0,
            step: "1",
        }],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "overlay", "#64748b",
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
    fn typical_price_is_the_average_mean_of_high_low_close() {
        // Typical prices 9, 12 and 15.
        let store = ohlc_store(&[(12.0, 6.0, 9.0), (15.0, 9.0, 12.0), (18.0, 12.0, 15.0)]);
        let values = typical_price_store(&store, 2, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert_eq!(&values[1..], &[10.5, 13.5]);
        assert_eq!(latest_typical_price_store(&store, 2), Some(13.5));
        assert_eq!(latest_typical_price_store(&store, 4), None);
        // A period of 1 is the plain typical price of each bar.
        assert_eq!(
            &*typical_price_store(&store, 1, &mut HashMap::new()),
            &[9.0, 12.0, 15.0]
        );
    }
}

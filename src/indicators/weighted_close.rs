use crate::indicators::kst::sma_from_series;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// (High + Low + 2 x Close) / 4 for bar `index`.
fn weighted_close_at(store: &CandleStore, index: usize) -> f64 {
    (store.high[index] + store.low[index] + 2.0 * store.close[index]) / 4.0
}

/// Weighted Close: the `period`-bar simple average of (High + Low + 2 x Close) / 4.
pub fn weighted_close_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("weighted_close:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let raw: Vec<f64> = (0..store.len())
        .map(|i| weighted_close_at(store, i))
        .collect();
    let rc = Rc::new(sma_from_series(&raw, period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

/// Latest Weighted Close, from the last `period` bars.
pub fn latest_weighted_close_store(store: &CandleStore, period: usize) -> Option<f64> {
    let len = store.len();
    if period == 0 || len < period {
        return None;
    }
    let sum: f64 = (len - period..len)
        .map(|i| weighted_close_at(store, i))
        .sum();
    Some(sum / period as f64)
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "WEIGHTED_CLOSE",
        name: "WEIGHTED CLOSE",
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
    fn weighted_close_is_the_average_weighted_mean() {
        // Weighted closes 9, 12 and 15.
        let store = ohlc_store(&[(12.0, 6.0, 9.0), (15.0, 9.0, 12.0), (18.0, 12.0, 15.0)]);
        let values = weighted_close_store(&store, 2, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert_eq!(&values[1..], &[10.5, 13.5]);
        assert_eq!(latest_weighted_close_store(&store, 2), Some(13.5));
        assert_eq!(latest_weighted_close_store(&store, 4), None);
        // A period of 1 is the plain weighted close of each bar.
        assert_eq!(
            &*weighted_close_store(&store, 1, &mut HashMap::new()),
            &[9.0, 12.0, 15.0]
        );
    }
}

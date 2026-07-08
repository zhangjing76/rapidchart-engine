use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Volume Chart: outputs volume as a line series.
pub fn volume_chart_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "vol_chart:v".to_string();
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let rc = Rc::new(store.volume.clone());
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_volume_chart_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 {
        return None;
    }
    Some(store.volume[store.len() - 1])
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "VOLUME_CHART",
        name: "VOLUME CHART",
        category: "Volume",
        pane: "separate",
        params: Vec::new(),
        outputs: vec![crate::descriptors::output_descriptor(
            "value",
            "histogram",
            "separate",
            "#94a3b8",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64, f64)]) -> CandleStore {
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
    fn volume_chart_is_the_raw_volume_series() {
        let store = ohlcv_store(&[(1.0, 1.0, 1.0, 2.0), (2.0, 2.0, 2.0, 3.0)]);
        let values = volume_chart_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[2.0, 3.0]);
        assert_eq!(latest_volume_chart_store(&store), Some(3.0));
    }

    #[test]
    fn volume_chart_preserves_varying_volumes() {
        let store = ohlcv_store(&[
            (1.0, 1.0, 1.0, 5.0),
            (2.0, 2.0, 2.0, 7.0),
            (3.0, 3.0, 3.0, 11.0),
        ]);
        let values = volume_chart_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[5.0, 7.0, 11.0]);
        assert_eq!(latest_volume_chart_store(&store), Some(11.0));
    }
}

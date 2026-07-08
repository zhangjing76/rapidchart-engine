use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn obv_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "obv:close:volume".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = Vec::with_capacity(store.len());
    let mut current = 0.0;
    for (index, (&close, &volume)) in store.close.iter().zip(store.volume.iter()).enumerate() {
        if index > 0 {
            let previous = store.close[index - 1];
            if close > previous {
                current += volume;
            } else if close < previous {
                current -= volume;
            }
        }
        out.push(current);
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_obv_store(store: &CandleStore, output: Option<&[f64]>) -> Option<f64> {
    let last = store.last_close()?;
    if store.len() == 1 {
        return Some(0.0);
    }
    let previous = output
        .and_then(|values| values.get(store.len() - 2))
        .copied()
        .and_then(nan_to_none)
        .unwrap_or(0.0);
    let previous_close = store.close[store.len() - 2];
    if last > previous_close {
        Some(previous + store.volume[store.len() - 1])
    } else if last < previous_close {
        Some(previous - store.volume[store.len() - 1])
    } else {
        Some(previous)
    }
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "OBV",
                name: "OBV",
                category: "Volume",
                pane: "separate",
                params: Vec::new(),
                outputs: vec![crate::descriptors::output_descriptor("value", "line", "separate", "#059669")],
            }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(close, _)| *close).collect(),
            values.iter().map(|(_, volume)| *volume).collect(),
        )
    }

    #[test]
    fn obv_tracks_directional_volume_flow() {
        let store = ohlcv_store(&[(10.0, 2.0), (12.0, 3.0), (11.0, 5.0), (11.0, 7.0)]);
        let values = obv_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[0.0, 3.0, -2.0, -2.0]);
        assert_eq!(latest_obv_store(&store, Some(&values[..])), Some(-2.0));
    }
}

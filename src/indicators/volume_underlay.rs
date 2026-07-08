use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Volume Underlay: outputs volume value, signed positive for up bars (close >= prev close)
/// and negative for down bars (close < prev close). The sign allows the renderer
/// to color the histogram green/red.
pub fn volume_underlay_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "vol_underlay:cv".to_string();
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len == 0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    out[0] = store.volume[0]; // first bar is neutral/positive
    for i in 1..len {
        if store.close[i] >= store.close[i - 1] {
            out[i] = store.volume[i]; // positive = up bar
        } else {
            out[i] = -store.volume[i]; // negative = down bar
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_volume_underlay_store(store: &CandleStore) -> Option<f64> {
    let len = store.len();
    if len == 0 {
        return None;
    }
    if len == 1 {
        return Some(store.volume[0]);
    }
    let i = len - 1;
    if store.close[i] >= store.close[i - 1] {
        Some(store.volume[i])
    } else {
        Some(-store.volume[i])
    }
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "VOLUME_UNDERLAY",
                name: "VOLUME UNDERLAY",
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
    fn volume_underlay_uses_sign_for_direction() {
        let store = ohlcv_store(&[
            (1.0, 1.0, 1.0, 2.0),
            (2.0, 2.0, 2.0, 3.0),
            (1.0, 1.0, 1.0, 4.0),
        ]);
        let values = volume_underlay_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[2.0, 3.0, -4.0]);
        assert_eq!(latest_volume_underlay_store(&store), Some(-4.0));
    }
}

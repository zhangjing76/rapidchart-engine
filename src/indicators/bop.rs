use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn bop_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "bop:ohlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let values: Vec<_> = (0..store.len())
        .map(|index| {
            let range = store.high[index] - store.low[index];
            if range == 0.0 {
                0.0
            } else {
                (store.close[index] - store.open[index]) / range
            }
        })
        .collect();
    let rc = Rc::new(values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_bop_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 {
        return None;
    }
    let i = store.len() - 1;
    let range = store.high[i] - store.low[i];
    Some(if range == 0.0 {
        0.0
    } else {
        (store.close[i] - store.open[i]) / range
    })
}

/// Balance of Power smoothed with an SMA over `period` bars (the raw value when period <= 1).
pub fn bop_smoothed_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let raw = bop_store(store, nodes);
    if period <= 1 {
        return raw;
    }
    let key = format!("bop:ohlc:sma:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(crate::indicators::kst::sma_from_series(&raw, period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_bop_smoothed_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period <= 1 {
        return latest_bop_store(store);
    }
    let len = store.len();
    if len < period {
        return None;
    }
    let sum: f64 = (len - period..len)
        .map(|i| {
            let range = store.high[i] - store.low[i];
            if range == 0.0 {
                0.0
            } else {
                (store.close[i] - store.open[i]) / range
            }
        })
        .sum();
    Some(sum / period as f64)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "BOP",
        name: "BOP",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "period",
            label: "Period",
            default: 14.0,
            min: 1.0,
            step: "1",
        }],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#9333ea",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(open, _, _, _, _)| *open).collect(),
            values.iter().map(|(_, high, _, _, _)| *high).collect(),
            values.iter().map(|(_, _, low, _, _)| *low).collect(),
            values.iter().map(|(_, _, _, close, _)| *close).collect(),
            values.iter().map(|(_, _, _, _, volume)| *volume).collect(),
        )
    }

    #[test]
    fn bop_is_open_to_close_relative_to_range() {
        let store = ohlcv_store(&[(1.0, 4.0, 0.0, 3.0, 1.0), (8.0, 8.0, 2.0, 5.0, 1.0)]);
        let values = bop_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[0.5, -0.5]);
        assert_eq!(latest_bop_store(&store), Some(-0.5));
    }
}

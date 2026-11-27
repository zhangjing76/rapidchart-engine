use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Trade Volume Index (TVI): a running total of volume signed by the price direction. A close
/// more than `min_tick` above the previous close sets the direction up, one more than `min_tick`
/// below sets it down, and a smaller move keeps the previous direction (none before the first
/// move). Starts at 0 on bar 0.
pub fn trade_volume_index_store(
    store: &CandleStore,
    min_tick: f64,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("tvi_tick:cv:{min_tick}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len > 0 {
        out[0] = 0.0;
    }
    let mut direction = 0.0;
    for i in 1..len {
        let change = store.close[i] - store.close[i - 1];
        if change > min_tick {
            direction = 1.0;
        } else if change < -min_tick {
            direction = -1.0;
        }
        out[i] = out[i - 1] + direction * store.volume[i];
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_trade_volume_index_store(store: &CandleStore, min_tick: f64) -> Option<f64> {
    trade_volume_index_store(store, min_tick, &mut HashMap::new())
        .last()
        .copied()
        .and_then(crate::nan_to_none)
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "TRADE_VOLUME_INDEX",
        name: "TRADE VOLUME INDEX",
        category: "Money Flow",
        pane: "separate",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "multiplier",
            label: "Min Tick Value",
            default: 0.5,
            min: 0.0,
            step: "0.01",
        }],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#0f766e",
        )],
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

    fn assert_series_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else {
                assert!((actual - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn trade_volume_index_accumulates_volume_by_direction() {
        let store = ohlcv_store(&[(1.0, 10.0), (2.0, 20.0), (1.0, 30.0), (1.0, 40.0)]);
        let values = trade_volume_index_store(&store, 0.5, &mut HashMap::new());

        // The unchanged last bar keeps the down direction.
        assert_series_close(&values, &[0.0, 20.0, -10.0, -50.0]);
        assert_eq!(latest_trade_volume_index_store(&store, 0.5), Some(-50.0));
    }

    #[test]
    fn moves_within_the_min_tick_keep_the_previous_direction() {
        let store = ohlcv_store(&[(10.0, 1.0), (10.2, 5.0), (11.0, 10.0), (10.8, 20.0)]);
        let values = trade_volume_index_store(&store, 0.5, &mut HashMap::new());

        // +0.2 sets no direction yet, +0.8 sets up, and -0.2 stays up.
        assert_series_close(&values, &[0.0, 0.0, 10.0, 30.0]);
    }
}

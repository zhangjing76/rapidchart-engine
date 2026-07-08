use crate::nan_to_none;
use crate::CandleStore;
use crate::NodeCache;

/// Anchored VWAP: VWAP starting from a user-specified bar index (anchor).
/// Before the anchor, values are NaN.
/// anchor = 0 means start from the first bar (equivalent to session VWAP).

pub fn anchored_vwap_store(
    store: &CandleStore,
    anchor: usize,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let len = store.len();
    let mut values = vec![f64::NAN; len];
    if anchor >= len {
        return vec![crate::named_series("value", values)];
    }
    let mut cum_pv = 0.0;
    let mut cum_vol = 0.0;
    for i in anchor..len {
        let tp = (store.high[i] + store.low[i] + store.close[i]) / 3.0;
        cum_pv += tp * store.volume[i];
        cum_vol += store.volume[i];
        if cum_vol > 0.0 {
            values[i] = cum_pv / cum_vol;
        }
    }
    vec![crate::named_series("value", values)]
}

/// For incremental update, recompute from anchor to current bar.
/// We store cumulative_pv and cumulative_volume as hidden state.
pub fn latest_anchored_vwap_store(
    store: &CandleStore,
    anchor: usize,
    outputs: &crate::types::IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let len = store.len();
    if len == 0 || anchor >= len {
        return (None, None, None);
    }
    let i = len - 1;
    let tp = (store.high[i] + store.low[i] + store.close[i]) / 3.0;

    let prev_pv = outputs
        .get("cumulative_pv")
        .and_then(|s| s.get(i.saturating_sub(1)).copied())
        .and_then(nan_to_none)
        .unwrap_or(0.0);
    let prev_vol = outputs
        .get("cumulative_volume")
        .and_then(|s| s.get(i.saturating_sub(1)).copied())
        .and_then(nan_to_none)
        .unwrap_or(0.0);

    // If we're before the anchor, zero
    if i < anchor {
        return (None, Some(0.0), Some(0.0));
    }

    let cum_pv = prev_pv + tp * store.volume[i];
    let cum_vol = prev_vol + store.volume[i];
    let value = if cum_vol > 0.0 {
        Some(cum_pv / cum_vol)
    } else {
        None
    };
    (value, Some(cum_pv), Some(cum_vol))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "ANCHORED_VWAP",
        name: "ANCHORED VWAP",
        category: "Volume",
        pane: "overlay",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "anchor",
            label: "Anchor Bar",
            default: 0.0,
            min: 0.0,
            step: "1",
        }],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "overlay", "#0f766e",
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
    fn anchored_vwap_is_the_running_typical_price_average() {
        let store = ohlcv_store(&[(3.0, 1.0, 2.0, 1.0), (5.0, 1.0, 3.0, 1.0)]);
        let outputs = anchored_vwap_store(&store, 0, &mut HashMap::new());

        assert_series_close(outputs[0].values.as_slice(), &[2.0, 2.5]);
    }
}

use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Market Facilitation Index: 10^scale x (high - low) / volume.
/// Measures price movement per unit of volume; the scale factor (6 by default) brings the
/// values to a readable size. Bars with no volume have no value.
fn facilitation(store: &CandleStore, scale: f64, index: usize) -> f64 {
    if store.volume[index] > 0.0 {
        10f64.powf(scale) * (store.high[index] - store.low[index]) / store.volume[index]
    } else {
        f64::NAN
    }
}

pub fn market_facilitation_store(
    store: &CandleStore,
    scale: f64,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("mfi_bw:hlv:{scale}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = (0..store.len())
        .map(|i| facilitation(store, scale, i))
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_market_facilitation_store(store: &CandleStore, scale: f64) -> Option<f64> {
    let index = store.len().checked_sub(1)?;
    Some(facilitation(store, scale, index)).filter(|v| !v.is_nan())
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "MARKET_FACILITATION",
        name: "MARKET FACILITATION INDEX",
        category: "Volume",
        pane: "separate",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "multiplier",
            label: "Scale Factor",
            default: 6.0,
            min: 0.0,
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

    fn ohlcv_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(_, _, volume)| *volume).collect(),
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
    fn market_facilitation_is_range_divided_by_volume() {
        let store = ohlcv_store(&[(10.0, 6.0, 2.0), (14.0, 8.0, 4.0)]);
        let values = market_facilitation_store(&store, 0.0, &mut HashMap::new());

        assert_series_close(&values, &[2.0, 1.5]);
        assert_eq!(latest_market_facilitation_store(&store, 0.0), Some(1.5));
    }

    #[test]
    fn market_facilitation_is_scaled_by_a_power_of_ten() {
        let store = ohlcv_store(&[(10.0, 6.0, 2_000_000.0), (14.0, 8.0, 0.0)]);
        let values = market_facilitation_store(&store, 6.0, &mut HashMap::new());

        // 10^6 x 4 / 2,000,000 = 2; the bar with no volume has no value.
        assert_series_close(&values, &[2.0, f64::NAN]);
        assert_eq!(latest_market_facilitation_store(&store, 6.0), None);
    }
}

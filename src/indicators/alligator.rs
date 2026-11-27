use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::rc::Rc;

/// Smoothed Moving Average (Wilder's smoothing): alpha = 1/period
fn smma_values(values: &[f64], period: usize) -> Series {
    let alpha = 1.0 / period as f64;
    let mut out = Vec::with_capacity(values.len());
    let mut current = None::<f64>;
    for &v in values {
        if v.is_nan() {
            out.push(f64::NAN);
        } else {
            let next = match current {
                Some(prev) => alpha * v + (1.0 - alpha) * prev,
                None => v,
            };
            current = Some(next);
            out.push(next);
        }
    }
    out
}

/// Bill Williams Alligator indicator.
/// - Jaw: SMMA(13) of median price, drawn 8 bars ahead
/// - Teeth: SMMA(8) of median price, drawn 5 bars ahead
/// - Lips: SMMA(5) of median price, drawn 3 bars ahead
///
/// Values are returned at the bar they were calculated on, like the Ichimoku leading spans,
/// so the last values are kept. `indicatorOutputShift` in engine.ts gives the forward shift
/// for drawing, which places the newest values past the last bar.

pub fn alligator_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    let key = "alligator:jaw";
    if let Some(jaw_rc) = nodes.get(key) {
        let jaw = (**jaw_rc).clone();
        let teeth = (**nodes.get("alligator:teeth").unwrap()).clone();
        let lips = (**nodes.get("alligator:lips").unwrap()).clone();
        return vec![
            crate::named_series("jaw", jaw),
            crate::named_series("teeth", teeth),
            crate::named_series("lips", lips),
        ];
    }
    let median: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(h, l)| (h + l) / 2.0)
        .collect();
    let jaw = smma_values(&median, 13);
    let teeth = smma_values(&median, 8);
    let lips = smma_values(&median, 5);
    nodes.insert("alligator:jaw".to_string(), Rc::new(jaw.clone()));
    nodes.insert("alligator:teeth".to_string(), Rc::new(teeth.clone()));
    nodes.insert("alligator:lips".to_string(), Rc::new(lips.clone()));
    vec![
        crate::named_series("jaw", jaw),
        crate::named_series("teeth", teeth),
        crate::named_series("lips", lips),
    ]
}

/// Latest value of each Alligator line, at the last bar (before the forward shift for drawing).
pub fn latest_alligator_store(store: &CandleStore) -> (Option<f64>, Option<f64>, Option<f64>) {
    if store.len() == 0 {
        return (None, None, None);
    }
    let median: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(h, l)| (h + l) / 2.0)
        .collect();
    let latest = |period: usize| {
        smma_values(&median, period)
            .last()
            .copied()
            .and_then(nan_to_none)
    };
    (latest(13), latest(8), latest(5))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "ALLIGATOR",
        name: "ALLIGATOR",
        category: "Averages/Bands",
        pane: "overlay",
        params: Vec::new(),
        outputs: vec![
            crate::descriptors::output_descriptor("jaw", "line", "overlay", "#2563eb"),
            crate::descriptors::output_descriptor("teeth", "line", "overlay", "#dc2626"),
            crate::descriptors::output_descriptor("lips", "line", "overlay", "#059669"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Bars each line is drawn ahead of its source bar (indicatorOutputShift in engine.ts).
    const JAW_SHIFT: usize = 8;
    const TEETH_SHIFT: usize = 5;
    const LIPS_SHIFT: usize = 3;

    fn ohlc_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            vec![1.0; len],
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
    fn alligator_is_flat_for_constant_prices() {
        let store = ohlc_store(&[10.0; 12]);
        let values = alligator_store(&store, &mut HashMap::new());

        // Unshifted: every bar has a value, including the last ones that are drawn
        // past the last bar.
        for output in &values {
            assert_series_close(&output.values, &[10.0; 12]);
        }
        let (jaw, teeth, lips) = latest_alligator_store(&store);
        assert!((jaw.unwrap() - 10.0).abs() < 1e-12);
        assert!((teeth.unwrap() - 10.0).abs() < 1e-12);
        assert!((lips.unwrap() - 10.0).abs() < 1e-12);
    }

    #[test]
    fn alligator_returns_smma_at_the_source_bar() {
        let store = ohlc_store(&[
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ]);
        let values = alligator_store(&store, &mut HashMap::new());

        // These are the values that are drawn at bar 11 once shifted by 8, 5 and 3 bars.
        assert!((values[0].values[11 - JAW_SHIFT] - 1.438324988620847).abs() < 1e-12);
        assert!((values[1].values[11 - TEETH_SHIFT] - 3.1415672302246094).abs() < 1e-12);
        assert!((values[2].values[11 - LIPS_SHIFT] - 5.671088640000001).abs() < 1e-12);

        // The newest values are kept rather than shifted off the end.
        assert!(values.iter().all(|output| !output.values[11].is_nan()));

        let latest = latest_alligator_store(&store);
        assert_eq!(latest.0, Some(values[0].values[11]));
        assert_eq!(latest.1, Some(values[1].values[11]));
        assert_eq!(latest.2, Some(values[2].values[11]));
    }
}

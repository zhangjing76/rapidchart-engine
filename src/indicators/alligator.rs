use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::rc::Rc;

/// Smoothed Moving Average (Wilder's smoothing, alpha = 1/period), seeded with the simple average
/// of the first `period` values, so the first value is at index `period - 1`. NaN inputs are
/// skipped and give NaN.
pub(crate) fn smma_values(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 {
        return out;
    }
    let alpha = 1.0 / period as f64;
    let mut seed_sum = 0.0;
    let mut seed_count = 0;
    let mut current = None::<f64>;
    for (index, &value) in values.iter().enumerate() {
        if value.is_nan() {
            continue;
        }
        match current {
            Some(previous) => {
                let next = alpha * value + (1.0 - alpha) * previous;
                current = Some(next);
                out[index] = next;
            }
            None => {
                seed_sum += value;
                seed_count += 1;
                if seed_count == period {
                    let seed = seed_sum / period as f64;
                    current = Some(seed);
                    out[index] = seed;
                }
            }
        }
    }
    out
}
/// Bill Williams Alligator indicator.
/// - Jaw: SMMA(13) of median price, drawn 8 bars ahead (each SMMA is seeded with a simple average)
/// - Teeth: SMMA(8) of median price, drawn 5 bars ahead
/// - Lips: SMMA(5) of median price, drawn 3 bars ahead
///
/// Values are returned at the bar they were calculated on, like the Ichimoku leading spans,
/// so the last values are kept. `indicatorOutputShift` in engine.ts gives the forward shift
/// for drawing, which places the newest values past the last bar.

pub fn alligator_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    let key = "alligator_seeded:jaw";
    if let Some(jaw_rc) = nodes.get(key) {
        let jaw = (**jaw_rc).clone();
        let teeth = (**nodes.get("alligator_seeded:teeth").unwrap()).clone();
        let lips = (**nodes.get("alligator_seeded:lips").unwrap()).clone();
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
    nodes.insert("alligator_seeded:jaw".to_string(), Rc::new(jaw.clone()));
    nodes.insert("alligator_seeded:teeth".to_string(), Rc::new(teeth.clone()));
    nodes.insert("alligator_seeded:lips".to_string(), Rc::new(lips.clone()));
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

    #[test]
    fn alligator_is_flat_for_constant_prices() {
        let store = ohlc_store(&[10.0; 13]);
        let values = alligator_store(&store, &mut HashMap::new());

        // Each line starts once its first `period` bars are in, and every bar after that has a
        // value, including the last ones that are drawn past the last bar.
        for (output, period) in values.iter().zip([13, 8, 5]) {
            assert!(output.values[..period - 1].iter().all(|v| v.is_nan()));
            assert!(output.values[period - 1..]
                .iter()
                .all(|v| (v - 10.0).abs() < 1e-12));
        }
        let (jaw, teeth, lips) = latest_alligator_store(&store);
        assert!((jaw.unwrap() - 10.0).abs() < 1e-12);
        assert!((teeth.unwrap() - 10.0).abs() < 1e-12);
        assert!((lips.unwrap() - 10.0).abs() < 1e-12);
    }

    #[test]
    fn alligator_seeds_each_smma_with_a_simple_average() {
        let store = ohlc_store(&(1..=14).map(f64::from).collect::<Vec<_>>());
        let values = alligator_store(&store, &mut HashMap::new());

        // Jaw: the average of 1..=13 is 7, then (7 * 12 + 14) / 13.
        assert!(values[0].values[11].is_nan());
        assert!((values[0].values[12] - 7.0).abs() < 1e-12);
        assert!((values[0].values[13] - 98.0 / 13.0).abs() < 1e-12);
        // Teeth and lips start at the averages of 1..=8 and 1..=5.
        assert!((values[1].values[7] - 4.5).abs() < 1e-12);
        assert!((values[2].values[4] - 3.0).abs() < 1e-12);

        // Values are kept at their source bar; drawing shifts them by 8, 5 and 3 bars.
        let latest = latest_alligator_store(&store);
        assert_eq!(latest.0, Some(values[0].values[13]));
        assert_eq!(latest.1, Some(values[1].values[13]));
        assert_eq!(latest.2, Some(values[2].values[13]));
    }
}

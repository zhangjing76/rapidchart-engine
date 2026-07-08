use crate::CandleStore;
use crate::NodeCache;
use std::collections::HashMap;

/// Darvas Box Theory:
/// - A new box top is established when a new high is made and confirmed
///   (3 consecutive bars not exceeding it).
/// - The box bottom is the lowest low during the confirmation period.
/// - The box remains until price breaks above the top (new box starts)
///   or breaks below the bottom (exit signal).
///
/// Outputs: top (upper box boundary), bottom (lower box boundary)

pub fn darvas_box_store(store: &CandleStore, _nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    let len = store.len();
    let mut top = vec![f64::NAN; len];
    let mut bottom = vec![f64::NAN; len];
    if len < 4 {
        return vec![
            crate::named_series("top", top),
            crate::named_series("bottom", bottom),
        ];
    }

    let mut box_top = f64::NAN;
    let mut box_bottom = f64::NAN;
    let mut high_candidate = store.high[0];
    let mut confirm_count = 0u32;
    let mut lowest_during_confirm = store.low[0];
    let mut box_active = false;

    for i in 1..len {
        if box_active {
            if store.high[i] > box_top {
                box_active = false;
                high_candidate = store.high[i];
                confirm_count = 0;
                lowest_during_confirm = store.low[i];
            }
            top[i] = box_top;
            bottom[i] = box_bottom;
        } else {
            if store.high[i] > high_candidate {
                high_candidate = store.high[i];
                confirm_count = 0;
                lowest_during_confirm = store.low[i];
            } else {
                confirm_count += 1;
                lowest_during_confirm = lowest_during_confirm.min(store.low[i]);
                if confirm_count >= 3 {
                    box_top = high_candidate;
                    box_bottom = lowest_during_confirm;
                    box_active = true;
                    top[i] = box_top;
                    bottom[i] = box_bottom;
                }
            }
        }
    }

    vec![
        crate::named_series("top", top),
        crate::named_series("bottom", bottom),
    ]
}

pub fn latest_darvas_box_store(store: &CandleStore) -> (Option<f64>, Option<f64>) {
    let outputs = darvas_box_store(store, &mut HashMap::new());
    let t = outputs[0]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    let b = outputs[1]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    (t, b)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "DARVAS_BOX",
                name: "DARVAS BOX",
                category: "Support/Resistance",
                pane: "overlay",
                params: Vec::new(),
                outputs: vec![
                    crate::descriptors::output_descriptor("top", "line", "overlay", "#059669"),
                    crate::descriptors::output_descriptor("bottom", "line", "overlay", "#dc2626"),
                ],
            }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[(f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, low)| *low).collect(),
            values.iter().map(|(high, _)| *high).collect(),
            values.iter().map(|(_, low)| *low).collect(),
            values.iter().map(|(_, low)| *low).collect(),
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
    fn darvas_box_confirms_after_three_non_breaking_bars() {
        let store = ohlc_store(&[(1.0, 0.0), (2.0, 0.0), (1.5, 0.0), (1.0, 0.0), (1.0, 0.0)]);
        let values = darvas_box_store(&store, &mut HashMap::new());

        assert_series_close(
            &values[0].values,
            &[f64::NAN, f64::NAN, f64::NAN, f64::NAN, 2.0],
        );
        assert_series_close(
            &values[1].values,
            &[f64::NAN, f64::NAN, f64::NAN, f64::NAN, 0.0],
        );
        assert_eq!(latest_darvas_box_store(&store), (Some(2.0), Some(0.0)));
    }
}

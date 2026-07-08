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

/// Shift a series forward by `shift` bars (pad with NaN at the front).
fn shift_forward(values: &[f64], shift: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    for i in 0..values.len().saturating_sub(shift) {
        out[i + shift] = values[i];
    }
    out
}

/// Bill Williams Alligator indicator.
/// - Jaw: SMMA(13) of median price, shifted 8 bars forward
/// - Teeth: SMMA(8) of median price, shifted 5 bars forward
/// - Lips: SMMA(5) of median price, shifted 3 bars forward

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
    let jaw_raw = smma_values(&median, 13);
    let teeth_raw = smma_values(&median, 8);
    let lips_raw = smma_values(&median, 5);
    let jaw = shift_forward(&jaw_raw, 8);
    let teeth = shift_forward(&teeth_raw, 5);
    let lips = shift_forward(&lips_raw, 3);
    nodes.insert("alligator:jaw".to_string(), Rc::new(jaw.clone()));
    nodes.insert("alligator:teeth".to_string(), Rc::new(teeth.clone()));
    nodes.insert("alligator:lips".to_string(), Rc::new(lips.clone()));
    vec![
        crate::named_series("jaw", jaw),
        crate::named_series("teeth", teeth),
        crate::named_series("lips", lips),
    ]
}

/// Incremental latest value for Alligator.
/// Since the Alligator uses shifted outputs, we recompute the SMMA at the
/// appropriate offset for each line.
pub fn latest_alligator_store(store: &CandleStore) -> (Option<f64>, Option<f64>, Option<f64>) {
    let len = store.len();
    if len == 0 {
        return (None, None, None);
    }
    let median: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(h, l)| (h + l) / 2.0)
        .collect();

    // Jaw: SMMA(13) shifted 8 — the latest jaw value comes from SMMA at index len-1-8
    let jaw = if len > 8 {
        let smma = smma_values(&median[..len - 8], 13);
        smma.last().copied().and_then(nan_to_none)
    } else {
        None
    };

    // Teeth: SMMA(8) shifted 5 — the latest teeth value comes from SMMA at index len-1-5
    let teeth = if len > 5 {
        let smma = smma_values(&median[..len - 5], 8);
        smma.last().copied().and_then(nan_to_none)
    } else {
        None
    };

    // Lips: SMMA(5) shifted 3 — the latest lips value comes from SMMA at index len-1-3
    let lips = if len > 3 {
        let smma = smma_values(&median[..len - 3], 5);
        smma.last().copied().and_then(nan_to_none)
    } else {
        None
    };

    (jaw, teeth, lips)
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

        assert_series_close(
            &values[0].values,
            &[
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                10.0,
                10.0,
                10.0,
                10.0,
            ],
        );
        assert_series_close(
            &values[1].values,
            &[
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                10.0,
                10.0,
                10.0,
                10.0,
                10.0,
                10.0,
                10.0,
            ],
        );
        assert_series_close(
            &values[2].values,
            &[
                f64::NAN,
                f64::NAN,
                f64::NAN,
                10.0,
                10.0,
                10.0,
                10.0,
                10.0,
                10.0,
                10.0,
                10.0,
                10.0,
            ],
        );
        let latest = latest_alligator_store(&store);
        let (jaw, teeth, lips) = latest;
        assert!((jaw.unwrap() - 10.0).abs() < 1e-12);
        assert!((teeth.unwrap() - 10.0).abs() < 1e-12);
        assert!((lips.unwrap() - 10.0).abs() < 1e-12);
    }

    #[test]
    fn alligator_uses_shifted_smma_on_rising_prices() {
        let store = ohlc_store(&[
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ]);
        let values = alligator_store(&store, &mut HashMap::new());

        assert!((values[0].values[11] - 1.438324988620847).abs() < 1e-12);
        assert!((values[1].values[11] - 3.1415672302246094).abs() < 1e-12);
        assert!((values[2].values[11] - 5.671088640000001).abs() < 1e-12);

        let latest = latest_alligator_store(&store);
        assert_eq!(latest.0, Some(values[0].values[11]));
        assert_eq!(latest.1, Some(values[1].values[11]));
        assert_eq!(latest.2, Some(values[2].values[11]));
    }
}

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

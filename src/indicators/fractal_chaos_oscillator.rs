use crate::indicators::fractal_chaos_bands::is_fractal;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Fractal Chaos Oscillator:
/// +1 when a fractal high is detected, -1 when a fractal low is detected, 0 otherwise.
/// Fractals are found as for the Fractal Chaos Bands, on the bar that confirms them; a fractal
/// low wins when both land on the same bar.
pub fn fractal_chaos_oscillator_store(store: &CandleStore, _nodes: &mut NodeCache) -> RcSeries {
    let key = "fco_ties:hl".to_string();
    if let Some(values) = _nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 5 {
        let rc = Rc::new(out);
        _nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in 4..len {
        let mid = i - 2;
        // A fractal low on the same bar as a fractal high takes precedence.
        out[i] = if is_fractal(&store.low, mid, -1.0) {
            -1.0
        } else if is_fractal(&store.high, mid, 1.0) {
            1.0
        } else {
            0.0
        };
    }
    let rc = Rc::new(out);
    _nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_fractal_chaos_oscillator_store(store: &CandleStore) -> Option<f64> {
    fractal_chaos_oscillator_store(store, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "FRACTAL_CHAOS_OSCILLATOR",
        name: "FRACTAL CHAOS OSCILLATOR",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: Vec::new(),
        outputs: vec![crate::descriptors::output_descriptor(
            "value",
            "histogram",
            "separate",
            "#2563eb",
        )],
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

    #[test]
    fn fractal_chaos_oscillator_flags_the_fractal_high() {
        let store = ohlc_store(&[(1.0, 1.0), (3.0, 1.0), (5.0, 1.0), (3.0, 1.0), (1.0, 1.0)]);
        let values = fractal_chaos_oscillator_store(&store, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!(values[2].is_nan());
        assert!(values[3].is_nan());
        assert!((values[4] - 1.0).abs() < 1e-12);
        assert_eq!(latest_fractal_chaos_oscillator_store(&store), Some(1.0));
    }
}

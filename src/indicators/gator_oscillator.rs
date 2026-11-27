use crate::indicators::alligator::alligator_store;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::collections::HashMap;

/// Bars each Alligator line is drawn ahead of its source bar.
const JAW_SHIFT: usize = 8;
const TEETH_SHIFT: usize = 5;
const LIPS_SHIFT: usize = 3;

/// The value of `line` drawn at bar `index` once it is shifted `shift` bars ahead.
fn drawn_at(line: &[f64], index: usize, shift: usize) -> f64 {
    index
        .checked_sub(shift)
        .map_or(f64::NAN, |source| line[source])
}

/// Gator Oscillator: the gaps between the Alligator lines as they are drawn, with the jaw, teeth
/// and lips shifted 8, 5 and 3 bars ahead.
/// Upper histogram = |jaw - teeth| (positive)
/// Lower histogram = -|teeth - lips| (negative)
pub fn gator_oscillator_store(
    store: &CandleStore,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let lines = alligator_store(store, nodes);
    let (jaw, teeth, lips) = (&lines[0].values, &lines[1].values, &lines[2].values);
    let len = store.len();
    let mut upper: Series = vec![f64::NAN; len];
    let mut lower: Series = vec![f64::NAN; len];
    for i in 0..len {
        let jaw = drawn_at(jaw, i, JAW_SHIFT);
        let teeth = drawn_at(teeth, i, TEETH_SHIFT);
        let lips = drawn_at(lips, i, LIPS_SHIFT);
        // NaN inputs leave NaN, so each histogram starts once both of its lines are drawn.
        upper[i] = (jaw - teeth).abs();
        lower[i] = -(teeth - lips).abs();
    }
    vec![
        crate::named_series("upper", upper),
        crate::named_series("lower", lower),
    ]
}

pub fn latest_gator_oscillator_store(store: &CandleStore) -> (Option<f64>, Option<f64>) {
    let outputs = gator_oscillator_store(store, &mut HashMap::new());
    let last = |index: usize| {
        outputs[index]
            .values
            .last()
            .copied()
            .and_then(crate::nan_to_none)
    };
    (last(0), last(1))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "GATOR_OSCILLATOR",
        name: "GATOR OSCILLATOR",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: Vec::new(),
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "histogram", "separate", "#059669"),
            crate::descriptors::output_descriptor("lower", "histogram", "separate", "#dc2626"),
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
    fn gator_oscillator_is_zero_for_constant_prices() {
        let store = ohlc_store(&[10.0; 22]);
        let values = gator_oscillator_store(&store, &mut HashMap::new());

        // The jaw is first drawn at bar 12 + 8 and the teeth at bar 7 + 5.
        assert!(values[0].values[19].is_nan());
        assert!(values[0].values[20..].iter().all(|v| v.abs() < 1e-12));
        assert!(values[1].values[11].is_nan());
        assert!(values[1].values[12..].iter().all(|v| v.abs() < 1e-12));
        let latest = latest_gator_oscillator_store(&store);
        assert!(latest.0.unwrap().abs() < 1e-12);
        assert!(latest.1.unwrap().abs() < 1e-12);
    }

    #[test]
    fn gator_oscillator_measures_the_gaps_between_the_drawn_lines() {
        let store = ohlc_store(&(1..=22).map(f64::from).collect::<Vec<_>>());
        let lines = alligator_store(&store, &mut HashMap::new());
        let values = gator_oscillator_store(&store, &mut HashMap::new());

        let (jaw, teeth, lips) = (&lines[0].values, &lines[1].values, &lines[2].values);
        assert!((values[0].values[21] - (jaw[13] - teeth[16]).abs()).abs() < 1e-12);
        assert!((values[1].values[21] + (teeth[16] - lips[18]).abs()).abs() < 1e-12);
        // At bar 20 the drawn jaw is its first value, the 1..=13 average of 7.
        assert!((values[0].values[20] - (7.0 - teeth[15]).abs()).abs() < 1e-12);
    }
}

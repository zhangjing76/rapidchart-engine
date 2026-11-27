use crate::indicators::prime_number_bands::{prime_above, prime_below};
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// +1 when the close is in the upper part of the bar's prime bands, -1 in the lower part, and 0
/// within `tolerance_pct` percent of the band width around the middle (half above, half below).
fn prime_number_signal(high: f64, low: f64, close: f64, tolerance_pct: f64) -> f64 {
    let top = prime_above(high);
    let bottom = prime_below(low);
    if top.is_nan() || bottom.is_nan() || close.is_nan() {
        return f64::NAN;
    }
    let tolerance = tolerance_pct * (top - bottom) / 100.0;
    // Twice the distance of the close below the middle of the bands.
    let skew = top + bottom - 2.0 * close;
    if skew < -tolerance {
        1.0
    } else if skew > tolerance {
        -1.0
    } else {
        0.0
    }
}

/// Prime Number Oscillator: where the close sits within the Prime Number Bands (see
/// [`prime_number_signal`]), with `tolerance_pct` setting the neutral zone around the middle.
pub fn prime_number_oscillator_store(
    store: &CandleStore,
    tolerance_pct: f64,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("pno_bands:hlc:{tolerance_pct}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let out: Vec<f64> = (0..store.len())
        .map(|i| prime_number_signal(store.high[i], store.low[i], store.close[i], tolerance_pct))
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_prime_number_oscillator_store(
    store: &CandleStore,
    tolerance_pct: f64,
) -> Option<f64> {
    let i = store.len().checked_sub(1)?;
    crate::nan_to_none(prime_number_signal(
        store.high[i],
        store.low[i],
        store.close[i],
        tolerance_pct,
    ))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "PRIME_NUMBER_OSCILLATOR",
        name: "PRIME NUMBER OSCILLATOR",
        category: "Trend Analysis",
        pane: "separate",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "multiplier",
            label: "Tolerance %",
            default: 5.0,
            min: 0.0,
            step: "0.1",
        }],
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

    fn ohlc_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            vec![1.0; len],
        )
    }

    #[test]
    fn prime_number_oscillator_reads_the_close_against_the_prime_bands() {
        // High 53.5 and low 47 give bands 59 and 43: middle 51, and 5% of the 16-point width puts
        // the neutral zone at 50.6..51.4.
        let store = ohlc_store(&[
            (53.5, 47.0, 52.0),
            (53.5, 47.0, 50.0),
            (53.5, 47.0, 51.2),
            (53.5, 47.0, 50.7),
        ]);
        let values = prime_number_oscillator_store(&store, 5.0, &mut HashMap::new());

        assert_eq!(&*values, &[1.0, -1.0, 0.0, 0.0]);
        assert_eq!(latest_prime_number_oscillator_store(&store, 5.0), Some(0.0));
        let no_tolerance = prime_number_oscillator_store(&store, 0.0, &mut HashMap::new());
        assert_eq!(&*no_tolerance, &[1.0, -1.0, 1.0, -1.0]);
    }
}

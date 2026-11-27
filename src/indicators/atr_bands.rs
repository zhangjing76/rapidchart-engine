use crate::indicators::atr::atr_store;
use crate::CandleStore;
use crate::NodeCache;

/// ATR Bands: close ± multiplier × ATR(period), with no middle line.
///
/// The bands are centred on each bar's close, so they follow price bar by bar. A channel
/// centred on a moving average is the Keltner Channel (KELTNER).

const ATR_STATE_SLOT: usize = 2;

pub fn atr_bands_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let atr_rc = atr_store(store, period, nodes);
    let len = store.len();
    let mut upper = vec![f64::NAN; len];
    let mut lower = vec![f64::NAN; len];
    for i in 0..len {
        let close = store.close[i];
        let a = atr_rc[i];
        if !close.is_nan() && !a.is_nan() {
            upper[i] = close + multiplier * a;
            lower[i] = close - multiplier * a;
        }
    }
    vec![
        crate::named_series("upper", upper),
        crate::named_series("lower", lower),
        crate::named_series("atr_state", atr_rc),
    ]
}

pub fn latest_atr_bands_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    outputs: &crate::types::IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    let close = store.close.last().copied().filter(|c| !c.is_nan());
    let atr_val =
        crate::indicators::atr::latest_atr_store(store, period, outputs.get_slot(ATR_STATE_SLOT));
    match (close, atr_val) {
        (Some(c), Some(a)) => (Some(c + multiplier * a), Some(c - multiplier * a)),
        _ => (None, None),
    }
}
pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "ATR_BANDS",
        name: "ATR BANDS",
        category: "Averages/Bands",
        pane: "overlay",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "period",
                label: "Period",
                default: 5.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "multiplier",
                label: "Multiplier",
                default: 3.0,
                min: 0.1,
                step: "0.1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "line", "overlay", "#0f766e"),
            crate::descriptors::output_descriptor("lower", "line", "overlay", "#0f766e"),
        ],
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
    fn atr_bands_is_the_close_plus_and_minus_the_atr_band() {
        let store = ohlc_store(&[
            (1.0, 1.0, 1.0),
            (2.0, 2.0, 2.0),
            (3.0, 3.0, 3.0),
            (4.0, 4.0, 4.0),
            (5.0, 5.0, 5.0),
        ]);
        let outputs = atr_bands_store(&store, 3, 2.0, &mut HashMap::new());
        let arena = crate::IndicatorArena::from_named_outputs(outputs.clone());

        let names: Vec<&str> = outputs.iter().map(|o| o.name.as_str()).collect();
        assert_eq!(names, ["upper", "lower", "atr_state"]);
        // ATR(3) is 1.0 from bar 3, so the bands sit 2.0 either side of each close.
        assert_series_close(
            outputs[0].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 6.0, 7.0],
        );
        assert_series_close(
            outputs[1].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 2.0, 3.0],
        );
        assert_series_close(
            outputs[2].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 1.0, 1.0],
        );
        assert_eq!(
            latest_atr_bands_store(&store, 3, 2.0, &arena),
            (Some(7.0), Some(3.0))
        );
    }
}

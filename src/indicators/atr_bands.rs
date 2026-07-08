use crate::indicators::atr::atr_store;
use crate::indicators::ema::ema_close_store;
use crate::series::rc_into_owned;
use crate::CandleStore;
use crate::NodeCache;

/// ATR Bands: EMA(close, period) ± multiplier * ATR(period)

const MIDDLE_SLOT: usize = 1;
const ATR_STATE_SLOT: usize = 3;

pub fn atr_bands_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let middle_rc = ema_close_store(store, period, nodes);
    let atr_rc = atr_store(store, period, nodes);
    let len = store.len();
    let mut upper = vec![f64::NAN; len];
    let mut lower = vec![f64::NAN; len];
    for i in 0..len {
        let m = middle_rc[i];
        let a = atr_rc[i];
        if !m.is_nan() && !a.is_nan() {
            upper[i] = m + multiplier * a;
            lower[i] = m - multiplier * a;
        }
    }
    let middle = rc_into_owned(middle_rc);
    vec![
        crate::named_series("upper", upper),
        crate::named_series("middle", middle),
        crate::named_series("lower", lower),
        crate::named_series("atr_state", atr_rc),
    ]
}

pub fn latest_atr_bands_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    outputs: &crate::types::IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    // Reuse existing EMA and ATR latest logic
    let middle =
        crate::indicators::ema::latest_ema_store(store, period, outputs.get_slot(MIDDLE_SLOT));
    let atr_val =
        crate::indicators::atr::latest_atr_store(store, period, outputs.get_slot(ATR_STATE_SLOT));
    match (middle, atr_val) {
        (Some(m), Some(a)) => (Some(m + multiplier * a), Some(m), Some(m - multiplier * a)),
        _ => (None, None, None),
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
    fn atr_bands_is_the_manual_ema_plus_atr_band() {
        let store = ohlc_store(&[
            (1.0, 1.0, 1.0),
            (2.0, 2.0, 2.0),
            (3.0, 3.0, 3.0),
            (4.0, 4.0, 4.0),
            (5.0, 5.0, 5.0),
        ]);
        let outputs = atr_bands_store(&store, 3, 2.0, &mut HashMap::new());
        let arena = crate::IndicatorArena::from_named_outputs(outputs.clone());

        assert_series_close(outputs[0].values.as_slice(), &[f64::NAN, f64::NAN, f64::NAN, 5.125, 6.0625]);
        assert_series_close(outputs[1].values.as_slice(), &[1.0, 1.5, 2.25, 3.125, 4.0625]);
        assert_series_close(outputs[2].values.as_slice(), &[f64::NAN, f64::NAN, f64::NAN, 1.125, 2.0625]);
        assert_series_close(outputs[3].values.as_slice(), &[f64::NAN, f64::NAN, f64::NAN, 1.0, 1.0]);
        assert_eq!(
            latest_atr_bands_store(&store, 3, 2.0, &arena),
            (Some(6.0625), Some(4.0625), Some(2.0625))
        );
    }
}

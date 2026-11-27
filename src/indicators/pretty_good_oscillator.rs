use crate::indicators::ema::ema_sma_seeded;
use crate::indicators::sma::sma_close_store;
use crate::indicators::true_range::true_range_series_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Pretty Good Oscillator: (close - SMA(close, period)) / EMA(true range, period), with the EMA
/// seeded by the simple average of its first `period` true ranges.
pub fn pretty_good_oscillator_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("pgo_ema_tr:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma = sma_close_store(store, period, nodes);
    // The true range needs a previous close, so it starts at bar 1.
    let mut true_range = (*true_range_series_store(store, nodes)).clone();
    if let Some(first) = true_range.first_mut() {
        *first = f64::NAN;
    }
    let atr = ema_sma_seeded(&true_range, period);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    for i in 0..len {
        let s = sma[i];
        let a = atr[i];
        if !s.is_nan() && !a.is_nan() && a.abs() > 1e-10 {
            out[i] = (store.close[i] - s) / a;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_pretty_good_oscillator_store(store: &CandleStore, period: usize) -> Option<f64> {
    pretty_good_oscillator_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "PRETTY_GOOD_OSCILLATOR",
        "PRETTY GOOD OSCILLATOR",
        "Momentum/Oscillator",
        "separate",
        14,
    )
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
    fn pretty_good_oscillator_is_zero_when_close_matches_the_sma() {
        let store = ohlc_store(&[(3.0, 1.0, 2.0), (3.0, 1.0, 2.0), (3.0, 1.0, 2.0)]);
        let values = pretty_good_oscillator_store(&store, 2, &mut HashMap::new());

        assert!(values.last().unwrap().abs() < 1e-12);
        assert!((latest_pretty_good_oscillator_store(&store, 2).unwrap() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn pretty_good_oscillator_divides_by_the_ema_of_true_range() {
        let store = ohlc_store(&[(10.0, 8.0, 9.0), (12.0, 9.0, 11.0), (13.0, 11.0, 12.0)]);
        let values = pretty_good_oscillator_store(&store, 2, &mut HashMap::new());

        // True ranges from bar 1 are 3 and 2, so the EMA starts at 2.5 on bar 2; the close of 12
        // is 0.5 above its two-bar average.
        assert!(values[1].is_nan());
        assert!((values[2] - 0.2).abs() < 1e-12);
    }
}

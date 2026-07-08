use crate::indicators::ema::ema_series;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Stochastic Momentum Index (SMI):
/// D = close - (highest_high + lowest_low) / 2 over period
/// SMI = 100 * EMA(EMA(D, smooth), smooth) / EMA(EMA(HL_range/2, smooth), smooth)
pub fn stochastic_momentum_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("smi:hlc:{}:{}", period, smooth);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let mut d_series = vec![f64::NAN; len];
    let mut hl_series = vec![f64::NAN; len];
    for i in period - 1..len {
        let hh = store.high[i + 1 - period..=i]
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = store.low[i + 1 - period..=i]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        d_series[i] = store.close[i] - (hh + ll) / 2.0;
        hl_series[i] = (hh - ll) / 2.0;
    }
    let d_smooth1 = ema_series(&d_series, smooth);
    let d_smooth2 = ema_series(&d_smooth1, smooth);
    let hl_smooth1 = ema_series(&hl_series, smooth);
    let hl_smooth2 = ema_series(&hl_smooth1, smooth);
    for i in 0..len {
        let d = d_smooth2[i];
        let hl = hl_smooth2[i];
        if !d.is_nan() && !hl.is_nan() && hl.abs() > 1e-10 {
            out[i] = (d / hl) * 100.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_stochastic_momentum_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
) -> Option<f64> {
    stochastic_momentum_store(store, period, smooth, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
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
    fn stochastic_momentum_is_hundred_when_close_stays_at_the_high() {
        let store = ohlc_store(&[
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
        ]);
        let values = stochastic_momentum_store(&store, 3, 2, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 100.0, 100.0]);
        assert_eq!(latest_stochastic_momentum_store(&store, 3, 2), Some(100.0));
    }
}

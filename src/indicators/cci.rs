use crate::indicators::derived::{hlc3_store, latest_hlc3};
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Typical price used by CCI, VWAP, and MFI.
pub fn typical_price_parts(high: f64, low: f64, close: f64) -> f64 {
    (high + low + close) / 3.0
}
/// Typical price at a given bar index.
pub fn typical_price_at(store: &CandleStore, index: usize) -> f64 {
    typical_price_parts(store.high[index], store.low[index], store.close[index])
}

/// CCI from the latest typical price, the window's average and its mean deviation. A window whose
/// typical prices are all the same has a mean deviation of rounding noise only; it gives 0 rather
/// than a ratio of two noise values.
fn cci_value(typical: f64, sma: f64, mean_deviation: f64) -> f64 {
    if mean_deviation <= 1e-10 * sma.abs() {
        0.0
    } else {
        (typical - sma) / (0.015 * mean_deviation)
    }
}

pub fn cci_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("cci:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let hlc3 = hlc3_store(store, nodes);
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
        let window = index + 1 - period..=index;
        let typical_prices: Vec<_> = window.clone().map(|i| hlc3[i]).collect();
        let sma = typical_prices.iter().sum::<f64>() / period as f64;
        let mean_deviation = typical_prices
            .iter()
            .map(|value| (value - sma).abs())
            .sum::<f64>()
            / period as f64;
        *item = cci_value(hlc3[index], sma, mean_deviation);
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_cci_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let hlc3_last = latest_hlc3(store)?;
    let typical_prices: Vec<_> = (start..store.len())
        .map(|i| (store.high[i] + store.low[i] + store.close[i]) / 3.0)
        .collect();
    let sma = typical_prices.iter().sum::<f64>() / period as f64;
    let mean_deviation = typical_prices
        .iter()
        .map(|value| (value - sma).abs())
        .sum::<f64>()
        / period as f64;
    Some(cci_value(hlc3_last, sma, mean_deviation))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("CCI", "CCI", "Momentum/Oscillator", "separate", 20)
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
    fn cci_is_zero_when_typical_price_matches_its_average() {
        let store = ohlc_store(&[(6.0, 4.0, 5.0), (6.0, 4.0, 5.0), (6.0, 4.0, 5.0)]);
        let values = cci_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 0.0]);
        assert_eq!(latest_cci_store(&store, 3), Some(0.0));
    }

    #[test]
    fn cci_is_zero_for_a_flat_window_despite_rounding() {
        // A typical price that cannot be represented exactly leaves rounding noise in the average.
        let bar = (5300.1, 5299.9, 5300.3);
        let store = ohlc_store(&[bar; 25]);
        let values = cci_store(&store, 20, &mut HashMap::new());
        assert!(values[19..].iter().all(|&v| v == 0.0));
        assert_eq!(latest_cci_store(&store, 20), Some(0.0));
    }
}

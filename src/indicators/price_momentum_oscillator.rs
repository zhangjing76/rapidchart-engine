use crate::indicators::ema::ema_sma_seeded;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Price Momentum Oscillator (PMO), DecisionPoint's double-smoothed rate of change:
/// 1. ROC = (close / close[1] - 1) * 1000 (the 1-bar percentage change, times 10)
/// 2. Smooth = ROC smoothed with a factor of 2 / period
/// 3. PMO = Smooth smoothed with a factor of 2 / smooth
///
/// A smoothing factor of 2 / n is an EMA over n - 1 periods, so both steps use simple-average
/// seeded EMAs over `period - 1` and `smooth - 1`; both must be at least 2.
pub fn price_momentum_oscillator_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("pmo_dp:close:{}:{}", period, smooth);
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let mut roc = vec![f64::NAN; len];
    for i in 1..len {
        if store.close[i - 1] != 0.0 {
            roc[i] = (store.close[i] / store.close[i - 1] - 1.0) * 1000.0;
        }
    }
    let smoothed = ema_sma_seeded(&roc, period.saturating_sub(1));
    let pmo = ema_sma_seeded(&smoothed, smooth.saturating_sub(1));
    out = pmo;
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_price_momentum_oscillator_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
) -> Option<f64> {
    price_momentum_oscillator_store(store, period, smooth, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "PRICE_MOMENTUM_OSCILLATOR",
        name: "PRICE MOMENTUM OSCILLATOR",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "period",
                label: "Smoothing",
                default: 35.0,
                min: 2.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "smooth",
                label: "Double Smooth",
                default: 20.0,
                min: 2.0,
                step: "1",
            },
        ],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#2563eb",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn close_store(values: &[f64]) -> CandleStore {
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
    fn pmo_is_zero_for_flat_prices() {
        let store = close_store(&[5.0, 5.0, 5.0, 5.0, 5.0]);
        let values = price_momentum_oscillator_store(&store, 3, 3, &mut HashMap::new());

        // ROC starts at bar 1, and each two-bar seed adds one more bar.
        assert_series_close(&values, &[f64::NAN, f64::NAN, f64::NAN, 0.0, 0.0]);
        assert_eq!(
            latest_price_momentum_oscillator_store(&store, 3, 3),
            Some(0.0)
        );
    }

    #[test]
    fn pmo_smooths_twice_with_a_factor_of_two_over_n() {
        let store = close_store(&[10.0, 11.0, 12.0, 13.0, 14.0]);
        let values = price_momentum_oscillator_store(&store, 3, 3, &mut HashMap::new());

        let roc: Vec<f64> = (1..5)
            .map(|i| (store.close[i] / store.close[i - 1] - 1.0) * 1000.0)
            .collect();
        // Period 3 smooths by 2/3, seeded with the average of the first two values.
        let factor = 2.0 / 3.0;
        let s2 = (roc[0] + roc[1]) / 2.0;
        let s3 = s2 + factor * (roc[2] - s2);
        let s4 = s3 + factor * (roc[3] - s3);
        let p3 = (s2 + s3) / 2.0;
        let p4 = p3 + factor * (s4 - p3);
        assert_series_close(&values, &[f64::NAN, f64::NAN, f64::NAN, p3, p4]);
        assert_eq!(
            latest_price_momentum_oscillator_store(&store, 3, 3),
            Some(values[4])
        );
    }
}

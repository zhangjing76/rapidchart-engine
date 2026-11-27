use crate::indicators::sma::sma_close_store;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Trend Intensity Index (M.H. Pee): each bar's deviation of the close from SMA(close, period),
/// over the last ceil(period / 2) bars. TII = 100 * up / (up + down), where up sums the
/// deviations at or above zero and down the absolute values of those below, so it runs 0..100.
/// It is 100 when there are no down deviations. The first value is at bar
/// period - 1 + ceil(period / 2) - 1.
pub fn trend_intensity_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("tii_pee:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma = sma_close_store(store, period, nodes);
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    let half = period.div_ceil(2);
    if period == 0 || len < period + half - 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let deviation: Vec<f64> = (0..len).map(|i| store.close[i] - sma[i]).collect();
    for i in period + half - 2..len {
        let (mut up, mut down) = (0.0, 0.0);
        for &change in &deviation[i + 1 - half..=i] {
            if change < 0.0 {
                down -= change;
            } else {
                up += change;
            }
        }
        out[i] = if down.abs() < 1e-8 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + up / down)
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_trend_intensity_store(store: &CandleStore, period: usize) -> Option<f64> {
    trend_intensity_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "TREND_INTENSITY",
        "TREND INTENSITY INDEX",
        "Momentum/Oscillator",
        "separate",
        14,
    )
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
    fn trend_intensity_is_100_when_no_close_is_below_its_sma() {
        let store = close_store(&[5.0, 5.0, 5.0, 5.0]);
        let values = trend_intensity_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, f64::NAN, 100.0]);
        assert_eq!(latest_trend_intensity_store(&store, 3), Some(100.0));
    }

    #[test]
    fn trend_intensity_is_the_up_share_of_the_last_half_period_deviations() {
        let store = close_store(&[1.0, 2.0, 3.0, 1.0, 5.0]);
        let values = trend_intensity_store(&store, 3, &mut HashMap::new());

        // Deviations from SMA(3) are +1, -1 and +2 at bars 2-4; each value uses the last two.
        assert_series_close(&values, &[f64::NAN, f64::NAN, f64::NAN, 50.0, 200.0 / 3.0]);
    }
}

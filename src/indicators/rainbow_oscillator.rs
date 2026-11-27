use crate::indicators::kst::sma_from_series;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Rainbow Oscillator (Mel Widner): 100 * (close - average of the 10 rainbow SMAs) / (highest
/// close - lowest close over the last `lookback` bars). The rainbow is 10 SMAs of `period`, each
/// taken of the one before, so the first value is at bar 10 * (period - 1).
pub fn rainbow_oscillator_store(
    store: &CandleStore,
    period: usize,
    lookback: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("rainbow_osc_range:close:{period}:{lookback}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || lookback == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let mut layers: Vec<Series> = Vec::with_capacity(10);
    layers.push(sma_from_series(&store.close, period));
    for i in 1..10 {
        layers.push(sma_from_series(&layers[i - 1], period));
    }
    for i in 0..len {
        let average = layers.iter().map(|layer| layer[i]).sum::<f64>() / 10.0;
        if average.is_nan() {
            continue;
        }
        let window = &store.close[(i + 1).saturating_sub(lookback)..=i];
        let highest = window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = window.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = highest - lowest;
        out[i] = if range > 1e-10 {
            100.0 * (store.close[i] - average) / range
        } else {
            0.0
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_rainbow_oscillator_store(
    store: &CandleStore,
    period: usize,
    lookback: usize,
) -> Option<f64> {
    rainbow_oscillator_store(store, period, lookback, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "RAINBOW_OSCILLATOR",
        name: "RAINBOW OSCILLATOR",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "period",
                label: "Period",
                default: 2.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "smooth",
                label: "HHV/LLV Lookback",
                default: 10.0,
                min: 1.0,
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
    fn rainbow_oscillator_is_zero_for_constant_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let values = rainbow_oscillator_store(&store, 1, 10, &mut HashMap::new());

        assert_eq!(&*values, &[0.0, 0.0, 0.0, 0.0]);
        assert_eq!(latest_rainbow_oscillator_store(&store, 1, 10), Some(0.0));
    }

    #[test]
    fn rainbow_oscillator_divides_by_the_close_range_over_the_lookback() {
        let store = close_store(&(0..12).map(f64::from).collect::<Vec<_>>());
        let values = rainbow_oscillator_store(&store, 2, 10, &mut HashMap::new());

        // On a line rising by 1 a bar, the k-th two-bar SMA trails the close by k/2, so the 10
        // layers average 2.75 below it. The last 10 closes span 9.
        let expected = 100.0 * 2.75 / 9.0;
        let mut want = vec![f64::NAN; 10];
        want.extend([expected, expected]);
        assert_series_close(&values, &want);
        assert_eq!(
            latest_rainbow_oscillator_store(&store, 2, 10),
            Some(values[11])
        );
    }
}

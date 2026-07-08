use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::rc::Rc;

const K_SLOT: usize = 0;

pub fn stochastic_k_values(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    for index in period - 1..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        let highest = window.iter().copied().fold(f64::MIN, f64::max);
        let lowest = window.iter().copied().fold(f64::MAX, f64::min);
        let range = highest - lowest;
        let current = values[index];
        out[index] = if range == 0.0 {
            0.0
        } else {
            100.0 * (current - lowest) / range
        };
    }
    out
}
pub fn smooth_series(values: &[f64], smooth: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if smooth == 0 {
        return out;
    }
    for (index, out_val) in out.iter_mut().enumerate() {
        if index + 1 < smooth {
            continue;
        }
        let window = &values[index + 1 - smooth..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        *out_val = window.iter().sum::<f64>() / smooth as f64;
    }
    out
}
pub fn stochastic_k_store(store: &CandleStore, period: usize) -> Series {
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return out;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
        let window = index + 1 - period..=index;
        let highest_high = window
            .clone()
            .map(|i| store.high[i])
            .fold(f64::MIN, f64::max);
        let lowest_low = window.map(|i| store.low[i]).fold(f64::MAX, f64::min);
        let range = highest_high - lowest_low;
        *item = if range == 0.0 {
            0.0
        } else {
            100.0 * (store.close[index] - lowest_low) / range
        };
    }
    out
}
pub fn stochastic_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let k = stochastic_k_store(store, period);
    let d = smooth_series(&k, smooth);
    let outputs = vec![crate::named_series("k", k), crate::named_series("d", d)];
    nodes.insert(
        format!("stoch:hlc:{period}:{smooth}"),
        Rc::clone(&outputs[0].values),
    );
    outputs
}
pub fn latest_stochastic_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None);
    }
    let start = store.len() - period;
    let highest_high = store.high[start..].iter().copied().fold(f64::MIN, f64::max);
    let lowest_low = store.low[start..].iter().copied().fold(f64::MAX, f64::min);
    let range = highest_high - lowest_low;
    let k = if range == 0.0 {
        0.0
    } else {
        100.0 * (store.close[store.len() - 1] - lowest_low) / range
    };
    if smooth == 0 || store.len() < period + smooth - 1 {
        return (Some(k), None);
    }
    let mut values = Vec::with_capacity(smooth);
    for index in store.len() - smooth..store.len() - 1 {
        let Some(value) = outputs.value_at_slot(K_SLOT, index) else {
            return (Some(k), None);
        };
        values.push(value);
    }
    values.push(k);
    (Some(k), Some(values.iter().sum::<f64>() / smooth as f64))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "STOCHASTIC",
        name: "STOCHASTIC",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "period",
                label: "Period",
                default: 14.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "smooth",
                label: "Smooth",
                default: 3.0,
                min: 1.0,
                step: "1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("k", "line", "separate", "#2563eb"),
            crate::descriptors::output_descriptor("d", "line", "separate", "#dc2626"),
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
    fn stochastic_k_and_d_are_one_hundred_for_close_at_the_high() {
        let store = ohlc_store(&[
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
            (2.0, 0.0, 2.0),
        ]);
        let values = stochastic_store(&store, 3, 2, &mut HashMap::new());

        assert_series_close(&values[0].values, &[f64::NAN, f64::NAN, 100.0, 100.0]);
        assert_series_close(&values[1].values, &[f64::NAN, f64::NAN, f64::NAN, 100.0]);
    }
}

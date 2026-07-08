use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Awesome Oscillator: SMA(5) of midpoint - SMA(34) of midpoint
/// where midpoint = (high + low) / 2
fn sma_of(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    let mut sum: f64 = values[..period].iter().sum();
    out[period - 1] = sum / period as f64;
    for i in period..values.len() {
        sum += values[i] - values[i - period];
        out[i] = sum / period as f64;
    }
    out
}

pub fn awesome_oscillator_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "ao:hl".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let midpoints: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(h, l)| (h + l) / 2.0)
        .collect();
    let sma5 = sma_of(&midpoints, 5);
    let sma34 = sma_of(&midpoints, 34);
    let out: Vec<f64> = sma5
        .iter()
        .zip(sma34.iter())
        .map(|(a, b)| {
            if a.is_nan() || b.is_nan() {
                f64::NAN
            } else {
                a - b
            }
        })
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_awesome_oscillator_store(store: &CandleStore) -> Option<f64> {
    awesome_oscillator_store(store, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "AWESOME_OSCILLATOR",
        name: "AWESOME OSCILLATOR",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: Vec::new(),
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

    fn hl_store(values: &[(f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            vec![1.0; len],
            values.iter().map(|(high, _)| *high).collect(),
            values.iter().map(|(_, low)| *low).collect(),
            vec![1.0; len],
            vec![1.0; len],
        )
    }

    #[test]
    fn awesome_oscillator_is_the_difference_of_two_midpoint_smas() {
        let bars = vec![(10.0, 0.0); 34];
        let store = hl_store(&bars);
        let values = awesome_oscillator_store(&store, &mut HashMap::new());

        assert!(values[..33].iter().all(|v| v.is_nan()));
        assert_eq!(values[33], 0.0);
        assert_eq!(latest_awesome_oscillator_store(&store), Some(0.0));
    }
}

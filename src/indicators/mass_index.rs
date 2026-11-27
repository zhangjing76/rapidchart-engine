use crate::indicators::ema::ema_sma_seeded;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Mass Index: SUM(EMA(H-L, 9) / EMA(EMA(H-L, 9), 9), period)
/// Default period = 25. Values > 27 suggest reversal ("bulge").
/// The EMAs start with a simple average, so the ratio starts at bar 16 and the sum at bar 15 + period.
pub fn mass_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("mass_sma:hl:{period}");
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 || period == 0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let hl: Vec<f64> = store
        .high
        .iter()
        .zip(store.low.iter())
        .map(|(h, l)| h - l)
        .collect();
    let ema1 = ema_sma_seeded(&hl, 9);
    let ema2 = ema_sma_seeded(&ema1, 9);
    // Ratio series
    let mut ratio = vec![f64::NAN; len];
    for i in 0..len {
        if !ema1[i].is_nan() && !ema2[i].is_nan() && ema2[i].abs() > 1e-10 {
            ratio[i] = ema1[i] / ema2[i];
        }
    }
    // Rolling sum of ratio over period
    for i in 0..len {
        if i + 1 < period {
            continue;
        }
        let window = &ratio[i + 1 - period..=i];
        let valid: Vec<f64> = window.iter().filter(|v| !v.is_nan()).copied().collect();
        if valid.len() == period {
            out[i] = valid.iter().sum();
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_mass_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    mass_index_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("MASS_INDEX", "MASS INDEX", "Volatility", "separate", 25)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.iter().map(|v| v + 1.0).collect(),
            values.to_vec(),
            values.to_vec(),
            vec![1.0; len],
        )
    }

    #[test]
    fn mass_index_is_the_period_sum_for_constant_ranges() {
        let store = ohlc_store(&[1.0; 30]);
        let values = mass_index_store(&store, 3, &mut HashMap::new());

        // The ratio of the two 9-bar averages is 1 from bar 16, so a 3-bar sum is 3 from bar 18.
        assert!(values[..18].iter().all(|v| v.is_nan()));
        assert!(values[18..].iter().all(|v| (v - 3.0).abs() < 1e-12));
        assert_eq!(latest_mass_index_store(&store, 3), Some(3.0));
    }

    #[test]
    fn mass_index_sums_the_ema_range_ratio() {
        let len = 40;
        let ranges: Vec<f64> = (0..len).map(|i| 1.0 + ((i * 7) % 5) as f64).collect();
        let store = CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            vec![10.0; len],
            ranges.iter().map(|r| 10.0 + r).collect(),
            vec![10.0; len],
            vec![10.0; len],
            vec![1.0; len],
        );
        let values = mass_index_store(&store, 5, &mut HashMap::new());

        let ema1 = ema_sma_seeded(&ranges, 9);
        let ema2 = ema_sma_seeded(&ema1, 9);
        let ratio: Vec<f64> = ema1.iter().zip(ema2.iter()).map(|(a, b)| a / b).collect();
        assert!(values[..20].iter().all(|v| v.is_nan()));
        for i in 20..len {
            let expected: f64 = ratio[i - 4..=i].iter().sum();
            assert!((values[i] - expected).abs() < 1e-12, "bar {i}");
        }
        assert!((latest_mass_index_store(&store, 5).unwrap() - values[len - 1]).abs() < 1e-12);
    }
}

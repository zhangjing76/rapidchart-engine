use crate::indicators::ema::ema_series;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Mass Index: SUM(EMA(H-L, 9) / EMA(EMA(H-L, 9), 9), period)
/// Default period = 25. Values > 27 suggest reversal ("bulge").
pub fn mass_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("mass:hl:{period}");
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
    let ema1 = ema_series(&hl, 9);
    let ema2 = ema_series(&ema1, 9);
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

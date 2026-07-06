use crate::NodeCache;
use crate::{CandleStore, IndicatorOutput};
use std::collections::HashMap;

/// Vortex Indicator:
/// VM+ = |High - prev_Low|, VM- = |Low - prev_High|
/// TR = max(H-L, |H-PC|, |L-PC|)
/// VI+ = SUM(VM+, period) / SUM(TR, period)
/// VI- = SUM(VM-, period) / SUM(TR, period)
pub fn vortex_indicator_store(store: &CandleStore, period: usize, _nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let len = store.len();
    let mut vi_plus = vec![f64::NAN; len];
    let mut vi_minus = vec![f64::NAN; len];
    if period == 0 || len < period + 1 {
        return vec![
            IndicatorOutput { name: "plus".to_string(), values: vi_plus },
            IndicatorOutput { name: "minus".to_string(), values: vi_minus },
        ];
    }
    let mut vm_plus = vec![0.0f64; len];
    let mut vm_minus = vec![0.0f64; len];
    let mut tr = vec![0.0f64; len];
    for i in 1..len {
        vm_plus[i] = (store.high[i] - store.low[i - 1]).abs();
        vm_minus[i] = (store.low[i] - store.high[i - 1]).abs();
        tr[i] = (store.high[i] - store.low[i])
            .max((store.high[i] - store.close[i - 1]).abs())
            .max((store.low[i] - store.close[i - 1]).abs());
    }
    for i in period..len {
        let sum_vmp: f64 = vm_plus[i + 1 - period..=i].iter().sum();
        let sum_vmm: f64 = vm_minus[i + 1 - period..=i].iter().sum();
        let sum_tr: f64 = tr[i + 1 - period..=i].iter().sum();
        if sum_tr > 1e-10 {
            vi_plus[i] = sum_vmp / sum_tr;
            vi_minus[i] = sum_vmm / sum_tr;
        }
    }
    vec![
        IndicatorOutput { name: "plus".to_string(), values: vi_plus },
        IndicatorOutput { name: "minus".to_string(), values: vi_minus },
    ]
}
pub fn latest_vortex_indicator_store(store: &CandleStore, period: usize) -> (Option<f64>, Option<f64>) {
    let o = vortex_indicator_store(store, period, &mut HashMap::new());
    let p = o[0].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    let m = o[1].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    (p, m)
}
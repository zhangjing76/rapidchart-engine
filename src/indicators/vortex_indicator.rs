use crate::CandleStore;
use crate::NodeCache;
use std::collections::HashMap;

/// Vortex Indicator:
/// VM+ = |High - prev_Low|, VM- = |Low - prev_High|
/// TR = max(H-L, |H-PC|, |L-PC|)
/// VI+ = SUM(VM+, period) / SUM(TR, period)
/// VI- = SUM(VM-, period) / SUM(TR, period)
pub fn vortex_indicator_store(
    store: &CandleStore,
    period: usize,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let len = store.len();
    let mut vi_plus = vec![f64::NAN; len];
    let mut vi_minus = vec![f64::NAN; len];
    if period == 0 || len < period + 1 {
        return vec![
            crate::named_series("plus", vi_plus),
            crate::named_series("minus", vi_minus),
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
        crate::named_series("plus", vi_plus),
        crate::named_series("minus", vi_minus),
    ]
}
pub fn latest_vortex_indicator_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>) {
    let o = vortex_indicator_store(store, period, &mut HashMap::new());
    let p = o[0]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    let m = o[1]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    (p, m)
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

    #[test]
    fn vortex_indicator_is_one_for_a_perfect_trend() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0]);
        let values = vortex_indicator_store(&store, 2, &mut HashMap::new());

        assert!(values[0].values[0].is_nan());
        assert!(values[1].values[0].is_nan());
        assert!((values[0].values[2] - 1.0).abs() < 1e-12);
        assert!((values[1].values[2] - 1.0).abs() < 1e-12);
        assert_eq!(
            latest_vortex_indicator_store(&store, 2),
            (Some(1.0), Some(1.0))
        );
    }
}

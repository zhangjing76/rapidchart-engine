use crate::indicators::ema::{ema_sma_seeded_close_store, latest_ema_sma_seeded_store};
use crate::CandleStore;
use crate::NodeCache;

/// Elder Ray Index:
/// Bull Power = High - EMA(close, period)
/// Bear Power = Low - EMA(close, period)
/// The EMA starts with a simple average, so both start at bar period - 1.

const EMA_STATE_SLOT: usize = 2;

pub fn elder_ray_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let ema = ema_sma_seeded_close_store(store, period, nodes);
    let len = store.len();
    let mut bull = vec![f64::NAN; len];
    let mut bear = vec![f64::NAN; len];
    for i in 0..len {
        if !ema[i].is_nan() {
            bull[i] = store.high[i] - ema[i];
            bear[i] = store.low[i] - ema[i];
        }
    }
    vec![
        crate::named_series("bull", bull),
        crate::named_series("bear", bear),
        crate::named_series("ema_state", ema),
    ]
}

pub fn latest_elder_ray_store(
    store: &CandleStore,
    period: usize,
    outputs: &crate::types::IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    let ema_val = latest_ema_sma_seeded_store(store, period, outputs.get_slot(EMA_STATE_SLOT));
    match ema_val {
        Some(e) => {
            let i = store.len() - 1;
            (Some(store.high[i] - e), Some(store.low[i] - e))
        }
        None => (None, None),
    }
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "ELDER_RAY",
        name: "ELDER RAY INDEX",
        category: "Momentum/Oscillator",
        pane: "separate",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "period",
            label: "Period",
            default: 13.0,
            min: 1.0,
            step: "1",
        }],
        outputs: vec![
            crate::descriptors::output_descriptor("bull", "histogram", "separate", "#059669"),
            crate::descriptors::output_descriptor("bear", "histogram", "separate", "#dc2626"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IndicatorArena;
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
    fn elder_ray_is_zero_for_constant_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let outputs = elder_ray_store(&store, 3, &mut HashMap::new());
        assert!(outputs[0].values[..2].iter().all(|v| v.is_nan()));
        assert_eq!(&outputs[0].values[2..], &[0.0, 0.0]);
        assert_eq!(&outputs[1].values[2..], &[0.0, 0.0]);

        let arena = IndicatorArena::from_named_outputs(outputs);
        assert_eq!(
            latest_elder_ray_store(&store, 3, &arena),
            (Some(0.0), Some(0.0))
        );
    }

    #[test]
    fn elder_ray_uses_high_and_low_distance_from_ema() {
        let store = CandleStore::from_raw_columns(
            vec![0, 1, 2],
            vec![10.0, 12.0, 14.0],
            vec![11.0, 13.0, 15.0],
            vec![9.0, 11.0, 13.0],
            vec![10.0, 12.0, 14.0],
            vec![1.0, 1.0, 1.0],
        );
        let outputs = elder_ray_store(&store, 3, &mut HashMap::new());

        // EMA(3) starts at bar 2 with (10 + 12 + 14) / 3 = 12.
        assert!(outputs[0].values[0].is_nan() && outputs[0].values[1].is_nan());
        assert_eq!(outputs[0].values[2], 3.0);
        assert_eq!(outputs[1].values[2], 1.0);

        let arena = IndicatorArena::from_named_outputs(outputs);
        assert_eq!(
            latest_elder_ray_store(&store, 3, &arena),
            (Some(3.0), Some(1.0))
        );
    }

    #[test]
    fn latest_elder_ray_matches_the_full_calculation() {
        let n = 40;
        let closes: Vec<f64> = (0..n).map(|i| 50.0 + ((i * 7) % 11) as f64).collect();
        let store_of = |len: usize| {
            CandleStore::from_raw_columns(
                (0..len as u32).collect(),
                closes[..len].to_vec(),
                closes[..len].iter().map(|c| c + 2.0).collect(),
                closes[..len].iter().map(|c| c - 3.0).collect(),
                closes[..len].to_vec(),
                vec![1.0; len],
            )
        };
        let full = elder_ray_store(&store_of(n), 13, &mut HashMap::new());
        for len in 1..=n {
            let previous = IndicatorArena::from_named_outputs(
                full.iter()
                    .map(|o| crate::named_series(&o.name, o.values[..len - 1].to_vec()))
                    .collect(),
            );
            let (bull, bear) = latest_elder_ray_store(&store_of(len), 13, &previous);
            for (got, slot) in [(bull, 0), (bear, 1)] {
                let expect = Some(full[slot].values[len - 1]).filter(|v| !v.is_nan());
                match (got, expect) {
                    (Some(a), Some(b)) => assert!((a - b).abs() < 1e-9, "len {len}"),
                    (a, b) => assert_eq!(a, b, "len {len}"),
                }
            }
        }
    }
}

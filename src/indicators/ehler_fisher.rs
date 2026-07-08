use crate::indicators::derived::hl2_store;
use crate::CandleStore;
use crate::NodeCache;
use std::collections::HashMap;

/// Ehlers Fisher Transform:
/// 1. Normalize price to -1..+1 range over period using (H+L)/2
/// 2. Apply Fisher Transform: fisher = 0.5 * ln((1+x)/(1-x))
/// 3. Outputs: fisher line and trigger (previous fisher value)

pub fn ehler_fisher_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let len = store.len();
    let hl2 = hl2_store(store, nodes);
    let mut fisher_out = vec![f64::NAN; len];
    let mut trigger_out = vec![f64::NAN; len];
    if period < 2 || len < period {
        return vec![
            crate::named_series("fisher", fisher_out),
            crate::named_series("trigger", trigger_out),
        ];
    }
    let mut prev_value = 0.0f64;
    let mut prev_fisher = 0.0f64;
    for i in period - 1..len {
        let max_high = store.high[i + 1 - period..=i]
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_low = store.low[i + 1 - period..=i]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let mid = hl2[i];
        let range = max_high - min_low;
        let normalized = if range > 1e-10 {
            0.33 * 2.0 * ((mid - min_low) / range - 0.5) + 0.67 * prev_value
        } else {
            0.67 * prev_value
        };
        let clamped = normalized.max(-0.999).min(0.999);
        let fisher = 0.5 * ((1.0 + clamped) / (1.0 - clamped)).ln() + 0.5 * prev_fisher;
        fisher_out[i] = fisher;
        trigger_out[i] = prev_fisher;
        prev_value = clamped;
        prev_fisher = fisher;
    }
    vec![
        crate::named_series("fisher", fisher_out),
        crate::named_series("trigger", trigger_out),
    ]
}

pub fn latest_ehler_fisher_store(store: &CandleStore, period: usize) -> (Option<f64>, Option<f64>) {
    let outputs = ehler_fisher_store(store, period, &mut HashMap::new());
    let fisher =
        outputs[0]
            .values
            .last()
            .copied()
            .and_then(|v| if v.is_nan() { None } else { Some(v) });
    let trigger =
        outputs[1]
            .values
            .last()
            .copied()
            .and_then(|v| if v.is_nan() { None } else { Some(v) });
    (fisher, trigger)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "EHLER_FISHER",
                name: "EHLER FISHER TRANSFORM",
                category: "Momentum/Oscillator",
                pane: "separate",
                params: vec![crate::descriptors::ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 10.0,
                    min: 2.0,
                    step: "1",
                }],
                outputs: vec![
                    crate::descriptors::output_descriptor("fisher", "line", "separate", "#2563eb"),
                    crate::descriptors::output_descriptor("trigger", "line", "separate", "#dc2626"),
                ],
            }
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
    fn ehler_fisher_is_zero_for_constant_prices() {
        let store = ohlc_store(&[10.0, 10.0, 10.0, 10.0]);
        let outputs = ehler_fisher_store(&store, 2, &mut HashMap::new());

        assert_series_close(&outputs[0].values, &[f64::NAN, 0.0, 0.0, 0.0]);
        assert_series_close(&outputs[1].values, &[f64::NAN, 0.0, 0.0, 0.0]);
        assert_eq!(latest_ehler_fisher_store(&store, 2), (Some(0.0), Some(0.0)));
    }

    #[test]
    fn ehler_fisher_matches_the_two_bar_normalization() {
        let store = ohlc_store(&[10.0, 20.0]);
        let outputs = ehler_fisher_store(&store, 2, &mut HashMap::new());
        let clamped = 0.33_f64;
        let expected_fisher = 0.5_f64 * ((1.0_f64 + clamped) / (1.0_f64 - clamped)).ln();

        assert_series_close(&outputs[0].values, &[f64::NAN, expected_fisher]);
        assert_series_close(&outputs[1].values, &[f64::NAN, 0.0]);
        assert_eq!(
            latest_ehler_fisher_store(&store, 2),
            (Some(expected_fisher), Some(0.0))
        );
    }
}

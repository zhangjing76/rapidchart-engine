use crate::indicators::atr::{atr_store, latest_atr_store};
use crate::indicators::bollinger::bollinger_outputs;
use crate::indicators::ema::{ema_sma_seeded_close_store, latest_ema_sma_seeded_store};
use crate::indicators::sma::{latest_sma_store, sma_close_store};
use crate::rc_into_owned;
use crate::CandleStore;
use crate::IndicatorArena;
use crate::NodeCache;
use std::rc::Rc;

const MIDDLE_SLOT: usize = 1;
const ATR_STATE_SLOT: usize = 3;

/// Keltner Channel: an EMA of the close (starting with a simple average) with bands `multiplier`
/// x ATR(period) above and below it. The bands start once the ATR has a value.
pub fn keltner_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let middle = rc_into_owned(ema_sma_seeded_close_store(store, period, nodes));
    let atr = rc_into_owned(atr_store(store, period, nodes));
    let mut upper = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    for ((upper_val, lower_val), (&mid, &atr_value)) in upper
        .iter_mut()
        .zip(lower.iter_mut())
        .zip(middle.iter().zip(atr.iter()))
    {
        if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        *upper_val = mid + multiplier * atr_value;
        *lower_val = mid - multiplier * atr_value;
    }
    let outputs = vec![
        crate::named_series("upper", upper),
        crate::named_series("middle", middle),
        crate::named_series("lower", lower),
        crate::named_series("atr_state", atr),
    ];
    for output in &outputs {
        nodes.insert(
            format!("keltner:{}:{}:{}", output.name, period, multiplier),
            Rc::clone(&output.values),
        );
    }
    outputs
}
pub fn latest_keltner_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_ema_sma_seeded_store(store, period, outputs.get_slot(MIDDLE_SLOT));
    let atr = latest_atr_store(store, period, outputs.get_slot(ATR_STATE_SLOT));
    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
}
/// STARC Bands: a `ma_period`-bar simple average of the close with bands `multiplier` x
/// ATR(period) above and below it. The bands start once the ATR has a value.
pub fn starc_store(
    store: &CandleStore,
    period: usize,
    ma_period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let middle = rc_into_owned(sma_close_store(store, ma_period, nodes));
    let atr = rc_into_owned(atr_store(store, period, nodes));
    let mut upper = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    for ((upper_val, lower_val), (&mid, &atr_value)) in upper
        .iter_mut()
        .zip(lower.iter_mut())
        .zip(middle.iter().zip(atr.iter()))
    {
        if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        *upper_val = mid + multiplier * atr_value;
        *lower_val = mid - multiplier * atr_value;
    }
    let mut outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!(
                "starc:{}:{}:{}:{}",
                output.name, period, ma_period, multiplier
            ),
            Rc::clone(&output.values),
        );
    }
    outputs.push(crate::named_series("atr_state", atr));
    outputs
}
pub fn latest_starc_store(
    store: &CandleStore,
    period: usize,
    ma_period: usize,
    multiplier: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_sma_store(store, ma_period);
    let atr = latest_atr_store(store, period, outputs.get_slot(ATR_STATE_SLOT));
    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
}
pub(crate) fn starc_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "STARC",
        name: "STARC",
        category: "Volatility",
        pane: "overlay",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "period",
                label: "Period",
                default: 15.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "smooth",
                label: "MA Period",
                default: 5.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "multiplier",
                label: "Multiplier",
                default: 1.3,
                min: 0.1,
                step: "0.1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "line", "overlay", "#0f766e"),
            crate::descriptors::output_descriptor("middle", "line", "overlay", "#2563eb"),
            crate::descriptors::output_descriptor("lower", "line", "overlay", "#0f766e"),
        ],
    }
}

pub(crate) fn keltner_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "KELTNER",
        name: "KELTNER",
        category: "Averages/Bands",
        pane: "overlay",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "period",
                label: "Period",
                default: 10.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "multiplier",
                label: "Multiplier",
                default: 5.0,
                min: 1.0,
                step: "0.1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "line", "overlay", "#0f766e"),
            crate::descriptors::output_descriptor("middle", "line", "overlay", "#2563eb"),
            crate::descriptors::output_descriptor("lower", "line", "overlay", "#0f766e"),
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
    fn keltner_is_the_manual_ema_plus_atr_band() {
        let store = ohlc_store(&[
            (1.0, 1.0, 1.0),
            (2.0, 2.0, 2.0),
            (3.0, 3.0, 3.0),
            (4.0, 4.0, 4.0),
            (5.0, 5.0, 5.0),
        ]);
        let outputs = keltner_store(&store, 3, 2.0, &mut HashMap::new());
        let arena = crate::IndicatorArena::from_named_outputs(outputs.clone());

        let nan = f64::NAN;
        // EMA(3) starts at bar 2 with (1 + 2 + 3) / 3; ATR(3) is 1 from bar 3.
        assert_series_close(outputs[0].values.as_slice(), &[nan, nan, nan, 5.0, 6.0]);
        assert_series_close(outputs[1].values.as_slice(), &[nan, nan, 2.0, 3.0, 4.0]);
        assert_series_close(outputs[2].values.as_slice(), &[nan, nan, nan, 1.0, 2.0]);
        assert_series_close(
            outputs[3].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 1.0, 1.0],
        );
        assert_eq!(
            latest_keltner_store(&store, 3, 2.0, &arena),
            (Some(6.0), Some(4.0), Some(2.0))
        );
    }

    #[test]
    fn latest_keltner_matches_the_full_calculation() {
        let bars: Vec<(f64, f64, f64)> = (0..40)
            .map(|i| {
                let mid = 50.0 + ((i * 7) % 11) as f64;
                (mid + 2.0, mid - 2.0, mid + ((i % 3) as f64 - 1.0))
            })
            .collect();
        let full = keltner_store(&ohlc_store(&bars), 10, 5.0, &mut HashMap::new());
        for len in 1..=bars.len() {
            let store = ohlc_store(&bars[..len]);
            let previous = crate::IndicatorArena::from_named_outputs(
                full.iter()
                    .map(|o| crate::named_series(&o.name, o.values[..len - 1].to_vec()))
                    .collect(),
            );
            let latest = latest_keltner_store(&store, 10, 5.0, &previous);
            for (slot, got) in [latest.0, latest.1, latest.2].into_iter().enumerate() {
                let expect = Some(full[slot].values[len - 1]).filter(|v| !v.is_nan());
                match (got, expect) {
                    (Some(a), Some(b)) => assert!((a - b).abs() < 1e-9, "len {len} slot {slot}"),
                    (a, b) => assert_eq!(a, b, "len {len} slot {slot}"),
                }
            }
        }
    }

    #[test]
    fn starc_centres_a_short_average_inside_atr_bands() {
        // Closes 1..10 with a 2-wide range: ATR(3) is 2 from bar 3.
        let bars: Vec<(f64, f64, f64)> = (1..=10)
            .map(|c| (c as f64 + 1.0, c as f64 - 1.0, c as f64))
            .collect();
        let outputs = starc_store(&ohlc_store(&bars), 3, 2, 1.5, &mut HashMap::new());
        let nan = f64::NAN;

        // The middle line is the 2-bar simple average of the close, from bar 1.
        assert_series_close(&outputs[1].values[..4], &[nan, 1.5, 2.5, 3.5]);
        // The bands start with the ATR, at 1.5 x 2 = 3 either side.
        assert_series_close(&outputs[0].values[..4], &[nan, nan, nan, 6.5]);
        assert_series_close(&outputs[2].values[..4], &[nan, nan, nan, 0.5]);
    }

    #[test]
    fn latest_starc_matches_the_full_calculation() {
        let bars: Vec<(f64, f64, f64)> = (0..40)
            .map(|i| {
                let mid = 50.0 + ((i * 7) % 11) as f64;
                (mid + 2.0, mid - 2.0, mid + ((i % 3) as f64 - 1.0))
            })
            .collect();
        let full = starc_store(&ohlc_store(&bars), 15, 5, 1.3, &mut HashMap::new());
        for len in 1..=bars.len() {
            let previous = crate::IndicatorArena::from_named_outputs(
                full.iter()
                    .map(|o| crate::named_series(&o.name, o.values[..len - 1].to_vec()))
                    .collect(),
            );
            let latest = latest_starc_store(&ohlc_store(&bars[..len]), 15, 5, 1.3, &previous);
            for (slot, got) in [latest.0, latest.1, latest.2].into_iter().enumerate() {
                let expect = Some(full[slot].values[len - 1]).filter(|v| !v.is_nan());
                match (got, expect) {
                    (Some(a), Some(b)) => assert!((a - b).abs() < 1e-9, "len {len} slot {slot}"),
                    (a, b) => assert_eq!(a, b, "len {len} slot {slot}"),
                }
            }
        }
    }
}

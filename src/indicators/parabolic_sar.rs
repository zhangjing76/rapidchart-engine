use crate::value_at_slice;
use crate::CandleStore;
use crate::IndicatorArena;
use crate::NodeCache;
use std::collections::HashMap;
use std::rc::Rc;

const VALUE_SLOT: usize = 0;
const EP_SLOT: usize = 1;
const AF_SLOT: usize = 2;
const TREND_SLOT: usize = 3;

/// SAR, extreme point, acceleration factor and trend (+1 up, -1 down) after a bar.
type SarState = (f64, f64, f64, f64);

/// Step the SAR on to bar `index`. The new SAR is checked against the bar's low (or high) before
/// it is held back to the previous two bars' lows (or highs), so a bar that reaches the projected
/// SAR reverses the trend.
fn next_sar(
    store: &CandleStore,
    index: usize,
    (sar, mut ep, mut af, mut trend): SarState,
    step: f64,
    max_step: f64,
) -> SarState {
    let mut next = sar + af * (ep - sar);
    if trend > 0.0 {
        if store.low[index] < next {
            trend = -1.0;
            next = ep;
            ep = store.low[index];
            af = step;
        } else {
            next = next.min(store.low[index - 1]).min(store.low[index - 2]);
            if store.high[index] > ep {
                ep = store.high[index];
                af = (af + step).min(max_step);
            }
        }
    } else if store.high[index] > next {
        trend = 1.0;
        next = ep;
        ep = store.high[index];
        af = step;
    } else {
        next = next.max(store.high[index - 1]).max(store.high[index - 2]);
        if store.low[index] < ep {
            ep = store.low[index];
            af = (af + step).min(max_step);
        }
    }
    (next, ep, af, trend)
}
pub fn parabolic_sar_store(
    store: &CandleStore,
    step: f64,
    max_step: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let key = format!("psar:ohlc:{step}:{max_step}");
    if let Some(values) = nodes.get(&key) {
        return vec![
            crate::named_series("value", Rc::clone(values)),
            crate::named_series(
                "ep",
                nodes
                    .get(&format!("psar:ep:{step}:{max_step}"))
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
            crate::named_series(
                "af",
                nodes
                    .get(&format!("psar:af:{step}:{max_step}"))
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
            crate::named_series(
                "trend",
                nodes
                    .get(&format!("psar:trend:{step}:{max_step}"))
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
        ];
    }
    let mut values = vec![f64::NAN; store.len()];
    let mut ep_values = vec![f64::NAN; store.len()];
    let mut af_values = vec![f64::NAN; store.len()];
    let mut trend_values = vec![f64::NAN; store.len()];
    if store.len() < 2 {
        return vec![
            crate::named_series("value", values),
            crate::named_series("ep", ep_values),
            crate::named_series("af", af_values),
            crate::named_series("trend", trend_values),
        ];
    }
    let mut trend = if store.close[1] >= store.close[0] {
        1.0
    } else {
        -1.0
    };
    let mut sar = if trend > 0.0 {
        store.low[0]
    } else {
        store.high[0]
    };
    let mut ep = if trend > 0.0 {
        store.high[1]
    } else {
        store.low[1]
    };
    let mut af = step;
    values[1] = sar;
    ep_values[1] = ep;
    af_values[1] = af;
    trend_values[1] = trend;
    for index in 2..store.len() {
        (sar, ep, af, trend) = next_sar(store, index, (sar, ep, af, trend), step, max_step);
        values[index] = sar;
        ep_values[index] = ep;
        af_values[index] = af;
        trend_values[index] = trend;
    }
    nodes.insert(key, Rc::new(values.clone()));
    nodes.insert(
        format!("psar:ep:{step}:{max_step}"),
        Rc::new(ep_values.clone()),
    );
    nodes.insert(
        format!("psar:af:{step}:{max_step}"),
        Rc::new(af_values.clone()),
    );
    nodes.insert(
        format!("psar:trend:{step}:{max_step}"),
        Rc::new(trend_values.clone()),
    );
    vec![
        crate::named_series("value", values),
        crate::named_series("ep", ep_values),
        crate::named_series("af", af_values),
        crate::named_series("trend", trend_values),
    ]
}
pub fn latest_parabolic_sar_store(
    store: &CandleStore,
    step: f64,
    max_step: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if store.len() < 2 {
        return (None, None, None, None);
    }
    if store.len() == 2 {
        let outputs = parabolic_sar_store(store, step, max_step, &mut HashMap::new());
        let index = store.len() - 1;
        return (
            value_at_slice(outputs[VALUE_SLOT].values.as_slice(), index),
            value_at_slice(outputs[EP_SLOT].values.as_slice(), index),
            value_at_slice(outputs[AF_SLOT].values.as_slice(), index),
            value_at_slice(outputs[TREND_SLOT].values.as_slice(), index),
        );
    }
    let previous_index = store.len() - 2;
    let previous_sar = outputs
        .value_at_slot(VALUE_SLOT, previous_index)
        .unwrap_or_else(|| {
            parabolic_sar_store(
                &CandleStore {
                    time: store.time[..store.len() - 1].to_vec(),
                    open: store.open[..store.len() - 1].to_vec(),
                    high: store.high[..store.len() - 1].to_vec(),
                    low: store.low[..store.len() - 1].to_vec(),
                    close: store.close[..store.len() - 1].to_vec(),
                    volume: store.volume[..store.len() - 1].to_vec(),
                },
                step,
                max_step,
                &mut HashMap::new(),
            )[VALUE_SLOT]
                .values[previous_index]
        });
    let previous_ep = outputs
        .value_at_slot(EP_SLOT, previous_index)
        .unwrap_or(previous_sar);
    let previous_af = outputs
        .value_at_slot(AF_SLOT, previous_index)
        .unwrap_or(step);
    let previous_trend = outputs
        .value_at_slot(TREND_SLOT, previous_index)
        .unwrap_or(1.0);
    let (sar, ep, af, trend) = next_sar(
        store,
        store.len() - 1,
        (previous_sar, previous_ep, previous_af, previous_trend),
        step,
        max_step,
    );
    (Some(sar), Some(ep), Some(af), Some(trend))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "PARABOLIC_SAR",
        name: "PARABOLIC SAR",
        category: "Trend Analysis",
        pane: "overlay",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "psar_step",
                label: "Step",
                default: 0.02,
                min: 0.001,
                step: "0.001",
            },
            crate::descriptors::ParamDescriptor {
                name: "psar_max_step",
                label: "Max",
                default: 0.2,
                min: 0.01,
                step: "0.01",
            },
        ],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "overlay", "#059669",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IndicatorArena;
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
    fn parabolic_sar_is_flat_for_constant_prices() {
        let store = ohlc_store(&[10.0, 10.0, 10.0, 10.0]);
        let outputs = parabolic_sar_store(&store, 0.02, 0.2, &mut HashMap::new());

        assert_series_close(&outputs[0].values, &[f64::NAN, 10.0, 10.0, 10.0]);
        assert_series_close(&outputs[1].values, &[f64::NAN, 10.0, 10.0, 10.0]);
        assert_series_close(&outputs[2].values, &[f64::NAN, 0.02, 0.02, 0.02]);
        assert_series_close(&outputs[3].values, &[f64::NAN, 1.0, 1.0, 1.0]);

        let arena = IndicatorArena::from_named_outputs(outputs);
        assert_eq!(
            latest_parabolic_sar_store(&store, 0.02, 0.2, &arena),
            (Some(10.0), Some(10.0), Some(0.02), Some(1.0))
        );
    }

    #[test]
    fn parabolic_sar_accelerates_upward_in_an_uptrend() {
        let store = ohlc_store(&[10.0, 11.0, 12.0, 13.0]);
        let outputs = parabolic_sar_store(&store, 0.02, 0.2, &mut HashMap::new());

        assert_series_close(&outputs[0].values, &[f64::NAN, 10.0, 10.0, 10.08]);
        assert_series_close(&outputs[1].values, &[f64::NAN, 11.0, 12.0, 13.0]);
        assert_series_close(&outputs[2].values, &[f64::NAN, 0.02, 0.04, 0.06]);
        assert_series_close(&outputs[3].values, &[f64::NAN, 1.0, 1.0, 1.0]);

        let arena = IndicatorArena::from_named_outputs(outputs);
        assert_eq!(
            latest_parabolic_sar_store(&store, 0.02, 0.2, &arena),
            (Some(10.08), Some(13.0), Some(0.06), Some(1.0))
        );
    }

    fn hl_store(values: &[(f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(h, l)| (h + l) / 2.0).collect(),
            values.iter().map(|(h, _)| *h).collect(),
            values.iter().map(|(_, l)| *l).collect(),
            values.iter().map(|(h, l)| (h + l) / 2.0).collect(),
            vec![1.0; len],
        )
    }

    #[test]
    fn a_bar_reaching_the_projected_sar_reverses_the_trend() {
        // Uptrend from bar 1: SAR 1, EP 12. Bar 2 makes a new high of 14 (AF 0.04) and the SAR is
        // held back to 1. Bar 3's projected SAR is 1 + 0.04 x 13 = 1.52, above its low of 1.4, so
        // the trend reverses, although the SAR held back to the previous two lows (1.3) would not
        // have been reached.
        let store = hl_store(&[(10.0, 1.0), (12.0, 1.5), (14.0, 1.3), (12.0, 1.4)]);
        let outputs = parabolic_sar_store(&store, 0.02, 0.2, &mut HashMap::new());
        assert_eq!(outputs[TREND_SLOT].values[2], 1.0);
        assert_eq!(outputs[TREND_SLOT].values[3], -1.0);
        // On reversal the SAR jumps to the extreme point of the old trend.
        assert_eq!(outputs[VALUE_SLOT].values[3], 14.0);
    }
    #[test]
    fn latest_parabolic_sar_matches_the_full_calculation() {
        let bars: Vec<(f64, f64)> = (0..80)
            .map(|i| {
                let mid = 50.0 + 6.0 * ((i as f64) / 7.0).sin() + ((i * 7) % 5) as f64 * 0.3;
                (mid + 1.0, mid - 1.0)
            })
            .collect();
        let full = parabolic_sar_store(&hl_store(&bars), 0.02, 0.2, &mut HashMap::new());
        for len in 3..=bars.len() {
            let previous = IndicatorArena::from_named_outputs(
                full.iter()
                    .map(|o| crate::named_series(&o.name, o.values[..len - 1].to_vec()))
                    .collect(),
            );
            let latest = latest_parabolic_sar_store(&hl_store(&bars[..len]), 0.02, 0.2, &previous);
            let got = [latest.0, latest.1, latest.2, latest.3];
            for (slot, got) in got.into_iter().enumerate() {
                let want = full[slot].values[len - 1];
                assert!((got.unwrap() - want).abs() < 1e-9, "len {len} slot {slot}");
            }
        }
    }
}

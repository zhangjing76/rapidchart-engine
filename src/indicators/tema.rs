use crate::indicators::ema::{ema_next, ema_sma_seeded, ema_sma_seeded_close_store};
use crate::rc_into_owned;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::collections::HashMap;
use std::rc::Rc;

const VALUE_SLOT: usize = 0;
const EMA1_SLOT: usize = 1;
const EMA2_SLOT: usize = 2;
const EMA3_SLOT: usize = 3;

type TemaResult = (Option<f64>, Option<f64>, Option<f64>, Option<f64>);

/// TEMA: 3 x EMA - 3 x EMA(EMA) + EMA(EMA(EMA)), from averages that start with a simple average,
/// so it starts at bar 3 x (period - 1). The three averages are kept as outputs so live updates
/// can step on from them.
pub fn tema_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let ema1 = rc_into_owned(ema_sma_seeded_close_store(store, period, nodes));
    let ema2_key = format!("tema_sma:ema2:{period}");
    let ema3_key = format!("tema_sma:ema3:{period}");
    let value_key = format!("tema_sma:value:{period}");
    if let (Some(ema2), Some(ema3), Some(value)) = (
        nodes.get(&ema2_key),
        nodes.get(&ema3_key),
        nodes.get(&value_key),
    ) {
        return vec![
            crate::named_series("value", Rc::clone(value)),
            crate::named_series("ema1", ema1),
            crate::named_series("ema2", Rc::clone(ema2)),
            crate::named_series("ema3", Rc::clone(ema3)),
        ];
    }
    let ema2 = ema_sma_seeded(&ema1, period);
    let ema3 = ema_sma_seeded(&ema2, period);
    let value: Series = ema1
        .iter()
        .zip(ema2.iter())
        .zip(ema3.iter())
        .map(|((first, second), third)| 3.0 * first - 3.0 * second + third)
        .collect();
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    nodes.insert(ema3_key, Rc::new(ema3.clone()));
    nodes.insert(value_key, Rc::new(value.clone()));
    vec![
        crate::named_series("value", value),
        crate::named_series("ema1", ema1),
        crate::named_series("ema2", ema2),
        crate::named_series("ema3", ema3),
    ]
}

/// Latest TEMA and its three averages. Steps on from the previous bar's averages when all three
/// have values; otherwise (while they are starting) calculates them from the closes.
pub fn latest_tema_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> TemaResult {
    let Some(last) = store.last_close() else {
        return (None, None, None, None);
    };
    let previous = store.len().checked_sub(2).and_then(|index| {
        Some((
            outputs.value_at_slot(EMA1_SLOT, index)?,
            outputs.value_at_slot(EMA2_SLOT, index)?,
            outputs.value_at_slot(EMA3_SLOT, index)?,
        ))
    });
    if let Some((ema1, ema2, ema3)) = previous {
        let ema1 = ema_next(last, ema1, period);
        let ema2 = ema_next(ema1, ema2, period);
        let ema3 = ema_next(ema2, ema3, period);
        let value = 3.0 * ema1 - 3.0 * ema2 + ema3;
        return (Some(value), Some(ema1), Some(ema2), Some(ema3));
    }
    let full = tema_store(store, period, &mut HashMap::new());
    let at = |slot: usize| full[slot].values.last().copied().filter(|v| !v.is_nan());
    (at(VALUE_SLOT), at(EMA1_SLOT), at(EMA2_SLOT), at(EMA3_SLOT))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor("TEMA", "TEMA", "Moving Average", "overlay", 50)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::named_series;

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

    fn assert_series_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan(), "expected NaN, got {actual}");
            } else {
                assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
            }
        }
    }

    #[test]
    fn tema_is_the_input_when_prices_are_constant() {
        let store = close_store(&[10.0; 8]);
        let outputs = tema_store(&store, 3, &mut HashMap::new());
        let nan = f64::NAN;
        // Period 3: the averages start at bars 2, 4 and 6.
        assert_series_close(
            &outputs[VALUE_SLOT].values,
            &[nan, nan, nan, nan, nan, nan, 10.0, 10.0],
        );
    }

    #[test]
    fn tema_combines_the_three_averages() {
        let store = close_store(&[10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0]);
        let outputs = tema_store(&store, 3, &mut HashMap::new());
        let nan = f64::NAN;
        // EMA(3) lags a straight line by 2: 12, 14, ... from bar 2. Each further EMA lags 2 more.
        assert_series_close(
            &outputs[EMA3_SLOT].values,
            &[nan, nan, nan, nan, nan, nan, 16.0, 18.0],
        );
        // 3 x EMA1 - 3 x EMA2 + EMA3 removes the lag on a straight line.
        assert_series_close(
            &outputs[VALUE_SLOT].values,
            &[nan, nan, nan, nan, nan, nan, 22.0, 24.0],
        );
    }

    #[test]
    fn latest_tema_matches_the_full_calculation() {
        let closes: Vec<f64> = (0..70).map(|i| 50.0 + ((i * 7) % 11) as f64).collect();
        for period in [1, 2, 3, 10, 20] {
            let full = tema_store(&close_store(&closes), period, &mut HashMap::new());
            for len in 1..=closes.len() {
                let store = close_store(&closes[..len]);
                let previous = IndicatorArena::from_named_outputs(
                    full.iter()
                        .map(|o| named_series(&o.name, o.values[..len - 1].to_vec()))
                        .collect(),
                );
                let latest = latest_tema_store(&store, period, &previous);
                let got = [latest.0, latest.1, latest.2, latest.3];
                for (slot, got) in got.into_iter().enumerate() {
                    let expect = Some(full[slot].values[len - 1]).filter(|v| !v.is_nan());
                    match (got, expect) {
                        (Some(a), Some(b)) => {
                            assert!((a - b).abs() < 1e-9, "period {period} len {len}")
                        }
                        (a, b) => assert_eq!(a, b, "period {period} len {len} slot {slot}"),
                    }
                }
            }
        }
    }
}

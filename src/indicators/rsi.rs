use crate::value_at_slice;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

const VALUE_SLOT: usize = 0;
const AVG_GAIN_SLOT: usize = 1;
const AVG_LOSS_SLOT: usize = 2;

pub fn rsi_outputs_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let key = format!("rsi:close:{period}");
    let gain_key = format!("rsi:avg_gain:{period}");
    let loss_key = format!("rsi:avg_loss:{period}");
    if let (Some(values), Some(avg_gains), Some(avg_losses)) =
        (nodes.get(&key), nodes.get(&gain_key), nodes.get(&loss_key))
    {
        return vec![
            crate::named_series("value", Rc::clone(values)),
            crate::named_series("avg_gain", Rc::clone(avg_gains)),
            crate::named_series("avg_loss", Rc::clone(avg_losses)),
        ];
    }
    let mut values = vec![f64::NAN; store.len()];
    let mut avg_gains = vec![f64::NAN; store.len()];
    let mut avg_losses = vec![f64::NAN; store.len()];
    if store.len() <= period {
        nodes.insert(key, Rc::new(values.clone()));
        nodes.insert(gain_key, Rc::new(avg_gains.clone()));
        nodes.insert(loss_key, Rc::new(avg_losses.clone()));
        return vec![
            crate::named_series("value", values),
            crate::named_series("avg_gain", avg_gains),
            crate::named_series("avg_loss", avg_losses),
        ];
    }
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    for index in 1..=period {
        let change = store.close[index] - store.close[index - 1];
        if change >= 0.0 {
            avg_gain += change;
        } else {
            avg_loss -= change;
        }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;
    values[period] = rsi_value(avg_gain, avg_loss);
    avg_gains[period] = avg_gain;
    avg_losses[period] = avg_loss;
    for index in period + 1..store.len() {
        let change = store.close[index] - store.close[index - 1];
        let gain = change.max(0.0);
        let loss = (-change).max(0.0);
        avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;
        values[index] = rsi_value(avg_gain, avg_loss);
        avg_gains[index] = avg_gain;
        avg_losses[index] = avg_loss;
    }
    nodes.insert(key, Rc::new(values.clone()));
    nodes.insert(gain_key, Rc::new(avg_gains.clone()));
    nodes.insert(loss_key, Rc::new(avg_losses.clone()));
    vec![
        crate::named_series("value", values),
        crate::named_series("avg_gain", avg_gains),
        crate::named_series("avg_loss", avg_losses),
    ]
}
pub fn rsi_value(avg_gain: f64, avg_loss: f64) -> f64 {
    if avg_loss == 0.0 {
        100.0
    } else {
        100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
    }
}
pub fn latest_rsi_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() <= period {
        return (None, None, None);
    }
    if store.len() == period + 1 {
        let outputs = rsi_outputs_store(store, period, &mut HashMap::new());
        let index = store.len() - 1;
        return (
            value_at_slice(outputs[0].values.as_slice(), index),
            value_at_slice(outputs[1].values.as_slice(), index),
            value_at_slice(outputs[2].values.as_slice(), index),
        );
    }
    let previous_index = store.len() - 2;
    let previous_outputs;
    let source_outputs = if outputs
        .value_at_slot(AVG_GAIN_SLOT, previous_index)
        .is_some()
        && outputs
            .value_at_slot(AVG_LOSS_SLOT, previous_index)
            .is_some()
    {
        outputs
    } else {
        let previous = CandleStore {
            time: store.time[..store.len() - 1].to_vec(),
            open: store.open[..store.len() - 1].to_vec(),
            high: store.high[..store.len() - 1].to_vec(),
            low: store.low[..store.len() - 1].to_vec(),
            close: store.close[..store.len() - 1].to_vec(),
            volume: store.volume[..store.len() - 1].to_vec(),
        };
        previous_outputs = IndicatorArena::from_named_outputs(rsi_outputs_store(
            &previous,
            period,
            &mut HashMap::new(),
        ));
        &previous_outputs
    };
    let previous_gain = source_outputs
        .value_at_slot(AVG_GAIN_SLOT, previous_index)
        .unwrap_or(0.0);
    let previous_loss = source_outputs
        .value_at_slot(AVG_LOSS_SLOT, previous_index)
        .unwrap_or(0.0);
    let change = store.close[store.len() - 1] - store.close[previous_index];
    let gain = change.max(0.0);
    let loss = (-change).max(0.0);
    let avg_gain = (previous_gain * (period - 1) as f64 + gain) / period as f64;
    let avg_loss = (previous_loss * (period - 1) as f64 + loss) / period as f64;
    (
        Some(rsi_value(avg_gain, avg_loss)),
        Some(avg_gain),
        Some(avg_loss),
    )
}
pub fn rsi_close_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("rsi:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = rsi_outputs_store(store, period, nodes)
        .remove(VALUE_SLOT)
        .values;
    nodes.insert(key, Rc::clone(&rc));
    rc
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
    fn rsi_is_the_manual_wilder_average() {
        let store = close_store(&[1.0, 2.0, 1.0, 3.0, 2.0]);
        let outputs = rsi_outputs_store(&store, 3, &mut HashMap::new());

        assert_series_close(outputs[0].values.as_slice(), &[f64::NAN, f64::NAN, f64::NAN, 75.0, 54.54545454545455]);
        assert_series_close(outputs[1].values.as_slice(), &[f64::NAN, f64::NAN, f64::NAN, 1.0, 0.6666666666666666]);
        assert_series_close(outputs[2].values.as_slice(), &[f64::NAN, f64::NAN, f64::NAN, 0.3333333333333333, 0.5555555555555555]);
        assert_eq!(
            latest_rsi_store(&store, 3, &crate::IndicatorArena::from_named_outputs(outputs.clone())),
            (Some(54.54545454545455), Some(0.6666666666666666), Some(0.5555555555555555))
        );
    }
}

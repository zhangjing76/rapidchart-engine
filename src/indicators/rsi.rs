use crate::IndicatorArena;
use crate::IndicatorOutput;
use crate::NodeCache;
use crate::{output_at, output_at_vec};
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

pub fn rsi_outputs_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let key = format!("rsi:close:{period}");
    let gain_key = format!("rsi:avg_gain:{period}");
    let loss_key = format!("rsi:avg_loss:{period}");
    if let (Some(values), Some(avg_gains), Some(avg_losses)) =
        (nodes.get(&key), nodes.get(&gain_key), nodes.get(&loss_key))
    {
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "avg_gain".to_string(),
                values: (**avg_gains).clone(),
            },
            IndicatorOutput {
                name: "avg_loss".to_string(),
                values: (**avg_losses).clone(),
            },
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
            IndicatorOutput {
                name: "value".to_string(),
                values,
            },
            IndicatorOutput {
                name: "avg_gain".to_string(),
                values: avg_gains,
            },
            IndicatorOutput {
                name: "avg_loss".to_string(),
                values: avg_losses,
            },
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
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "avg_gain".to_string(),
            values: avg_gains,
        },
        IndicatorOutput {
            name: "avg_loss".to_string(),
            values: avg_losses,
        },
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
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "avg_gain", index),
            output_at_vec(&outputs, "avg_loss", index),
        );
    }
    let previous_index = store.len() - 2;
    let previous_outputs;
    let source_outputs = if output_at(outputs, "avg_gain", previous_index).is_some()
        && output_at(outputs, "avg_loss", previous_index).is_some()
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
        previous_outputs =
            IndicatorArena::from_outputs(rsi_outputs_store(&previous, period, &mut HashMap::new()));
        &previous_outputs
    };
    let previous_gain = output_at(source_outputs, "avg_gain", previous_index).unwrap_or(0.0);
    let previous_loss = output_at(source_outputs, "avg_loss", previous_index).unwrap_or(0.0);
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
    let rc = Rc::new(rsi_outputs_store(store, period, nodes).remove(0).values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

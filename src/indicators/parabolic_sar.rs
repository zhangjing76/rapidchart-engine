use crate::IndicatorArena;
use crate::IndicatorOutput;
use crate::NodeCache;
use crate::{output_at, output_at_vec};
use crate::CandleStore;
use std::collections::HashMap;
use std::rc::Rc;

pub fn parabolic_sar_store(
    store: &CandleStore,
    step: f64,
    max_step: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let key = format!("psar:ohlc:{step}:{max_step}");
    if let Some(values) = nodes.get(&key) {
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "ep".to_string(),
                values: nodes
                    .get(&format!("psar:ep:{step}:{max_step}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "af".to_string(),
                values: nodes
                    .get(&format!("psar:af:{step}:{max_step}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "trend".to_string(),
                values: nodes
                    .get(&format!("psar:trend:{step}:{max_step}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
        ];
    }
    let mut values = vec![f64::NAN; store.len()];
    let mut ep_values = vec![f64::NAN; store.len()];
    let mut af_values = vec![f64::NAN; store.len()];
    let mut trend_values = vec![f64::NAN; store.len()];
    if store.len() < 2 {
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values,
            },
            IndicatorOutput {
                name: "ep".to_string(),
                values: ep_values,
            },
            IndicatorOutput {
                name: "af".to_string(),
                values: af_values,
            },
            IndicatorOutput {
                name: "trend".to_string(),
                values: trend_values,
            },
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
        let mut next_sar = sar + af * (ep - sar);
        if trend > 0.0 {
            next_sar = next_sar.min(store.low[index - 1]).min(store.low[index - 2]);
            if store.low[index] < next_sar {
                trend = -1.0;
                next_sar = ep;
                ep = store.low[index];
                af = step;
            } else if store.high[index] > ep {
                ep = store.high[index];
                af = (af + step).min(max_step);
            }
        } else {
            next_sar = next_sar
                .max(store.high[index - 1])
                .max(store.high[index - 2]);
            if store.high[index] > next_sar {
                trend = 1.0;
                next_sar = ep;
                ep = store.high[index];
                af = step;
            } else if store.low[index] < ep {
                ep = store.low[index];
                af = (af + step).min(max_step);
            }
        }
        sar = next_sar;
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
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "ep".to_string(),
            values: ep_values,
        },
        IndicatorOutput {
            name: "af".to_string(),
            values: af_values,
        },
        IndicatorOutput {
            name: "trend".to_string(),
            values: trend_values,
        },
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
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "ep", index),
            output_at_vec(&outputs, "af", index),
            output_at_vec(&outputs, "trend", index),
        );
    }
    let previous_index = store.len() - 2;
    let previous_sar = output_at(outputs, "value", previous_index).unwrap_or_else(|| {
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
        )[0]
        .values[previous_index]
    });
    let previous_ep = output_at(outputs, "ep", previous_index).unwrap_or(previous_sar);
    let previous_af = output_at(outputs, "af", previous_index).unwrap_or(step);
    let previous_trend = output_at(outputs, "trend", previous_index).unwrap_or(1.0);
    let index = store.len() - 1;
    let mut trend = previous_trend;
    let mut sar = previous_sar + previous_af * (previous_ep - previous_sar);
    let mut ep = previous_ep;
    let mut af = previous_af;
    if trend > 0.0 {
        sar = sar.min(store.low[index - 1]).min(store.low[index - 2]);
        if store.low[index] < sar {
            trend = -1.0;
            sar = previous_ep;
            ep = store.low[index];
            af = step;
        } else if store.high[index] > ep {
            ep = store.high[index];
            af = (af + step).min(max_step);
        }
    } else {
        sar = sar.max(store.high[index - 1]).max(store.high[index - 2]);
        if store.high[index] > sar {
            trend = 1.0;
            sar = previous_ep;
            ep = store.high[index];
            af = step;
        } else if store.low[index] < ep {
            ep = store.low[index];
            af = (af + step).min(max_step);
        }
    }
    (Some(sar), Some(ep), Some(af), Some(trend))
}
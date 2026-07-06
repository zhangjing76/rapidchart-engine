use crate::indicators::ema::{ema_close, ema_close_store, ema_next, ema_series, ema_values};
use crate::rc_into_owned;
use crate::IndicatorArena;
use crate::IndicatorOutput;
use crate::MacdParams;
use crate::NodeCache;
use crate::{output_at, output_at_vec};
use crate::{Bar, CandleStore, Series};
use std::collections::HashMap;
use std::rc::Rc;

#[allow(dead_code)]
type MacdResult = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

#[allow(dead_code)]
pub fn macd(bars: &[Bar], params: MacdParams, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let fast = ema_close(bars, params.fast, nodes);
    let slow = ema_close(bars, params.slow, nodes);
    let macd_line: Vec<_> = fast
        .iter()
        .zip(slow.iter())
        .map(|(fast, slow)| *fast - *slow)
        .collect();
    let signal = ema_values(macd_line.iter().copied(), params.signal);
    let histogram: Vec<_> = macd_line
        .iter()
        .zip(signal.iter())
        .map(|(macd, signal)| *macd - *signal)
        .collect();
    vec![
        IndicatorOutput {
            name: "macd".to_string(),
            values: macd_line,
        },
        IndicatorOutput {
            name: "signal".to_string(),
            values: signal,
        },
        IndicatorOutput {
            name: "histogram".to_string(),
            values: histogram,
        },
        IndicatorOutput {
            name: "fast_ema".to_string(),
            values: fast,
        },
        IndicatorOutput {
            name: "slow_ema".to_string(),
            values: slow,
        },
    ]
}
pub fn ppo(bars: &[Bar], params: MacdParams, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let fast = ema_close(bars, params.fast, nodes);
    let slow = ema_close(bars, params.slow, nodes);
    let ppo_line: Vec<_> = fast
        .iter()
        .zip(slow.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() && *slow != 0.0 => {
                100.0 * (*fast - *slow) / *slow
            }
            (a, b) if !a.is_nan() && !b.is_nan() => 0.0,
            _ => f64::NAN,
        })
        .collect();
    let signal = ema_series(&ppo_line, params.signal);
    let histogram: Vec<_> = ppo_line
        .iter()
        .zip(signal.iter())
        .map(|(ppo, signal)| match (ppo, signal) {
            (ppo, signal) if !ppo.is_nan() && !signal.is_nan() => *ppo - *signal,
            _ => f64::NAN,
        })
        .collect();
    vec![
        IndicatorOutput {
            name: "ppo".to_string(),
            values: ppo_line.clone(),
        },
        IndicatorOutput {
            name: "signal".to_string(),
            values: signal,
        },
        IndicatorOutput {
            name: "histogram".to_string(),
            values: histogram,
        },
        IndicatorOutput {
            name: "fast_ema".to_string(),
            values: fast,
        },
        IndicatorOutput {
            name: "slow_ema".to_string(),
            values: slow,
        },
    ]
}
#[allow(dead_code)]
pub fn latest_macd(bars: &[Bar], params: MacdParams, outputs: &IndicatorArena) -> MacdResult {
    let last = match bars.last() {
        Some(last) => last,
        None => return (None, None, None, None, None),
    };
    if bars.len() == 1 {
        return (
            Some(0.0),
            Some(0.0),
            Some(0.0),
            Some(last.close),
            Some(last.close),
        );
    }
    let previous_index = bars.len() - 2;
    let previous_close = bars[previous_index].close;
    let previous_fast = output_at(outputs, "fast_ema", previous_index).unwrap_or(previous_close);
    let previous_slow = output_at(outputs, "slow_ema", previous_index).unwrap_or(previous_close);
    let fast = ema_next(last.close, previous_fast, params.fast);
    let slow = ema_next(last.close, previous_slow, params.slow);
    let macd_line = fast - slow;
    let previous_signal = output_at(outputs, "signal", previous_index).unwrap_or(0.0);
    let signal = ema_next(macd_line, previous_signal, params.signal);
    let histogram = macd_line - signal;
    (
        Some(macd_line),
        Some(signal),
        Some(histogram),
        Some(fast),
        Some(slow),
    )
}
pub fn macd_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let macd_key = format!(
        "macd:value:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let signal_key = format!(
        "macd:signal:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let histogram_key = format!(
        "macd:histogram:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let fast_key = format!("ema:close:{}", params.fast);
    let slow_key = format!("ema:close:{}", params.slow);
    if let (Some(macd), Some(signal), Some(histogram), Some(fast_ema), Some(slow_ema)) = (
        nodes.get(&macd_key),
        nodes.get(&signal_key),
        nodes.get(&histogram_key),
        nodes.get(&fast_key),
        nodes.get(&slow_key),
    ) {
        return vec![
            IndicatorOutput {
                name: "macd".to_string(),
                values: (**macd).clone(),
            },
            IndicatorOutput {
                name: "signal".to_string(),
                values: (**signal).clone(),
            },
            IndicatorOutput {
                name: "histogram".to_string(),
                values: (**histogram).clone(),
            },
            IndicatorOutput {
                name: "fast_ema".to_string(),
                values: (**fast_ema).clone(),
            },
            IndicatorOutput {
                name: "slow_ema".to_string(),
                values: (**slow_ema).clone(),
            },
        ];
    }
    let fast_ema = rc_into_owned(ema_close_store(store, params.fast, nodes));
    let slow_ema = rc_into_owned(ema_close_store(store, params.slow, nodes));
    let macd: Series = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() => *fast - *slow,
            _ => f64::NAN,
        })
        .collect();
    let signal = ema_series(&macd, params.signal);
    let histogram: Series = macd
        .iter()
        .zip(signal.iter())
        .map(|(macd, signal)| match (macd, signal) {
            (macd, signal) if !macd.is_nan() && !signal.is_nan() => *macd - *signal,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(macd_key, Rc::new(macd.clone()));
    nodes.insert(signal_key, Rc::new(signal.clone()));
    nodes.insert(histogram_key, Rc::new(histogram.clone()));
    vec![
        IndicatorOutput {
            name: "macd".to_string(),
            values: macd,
        },
        IndicatorOutput {
            name: "signal".to_string(),
            values: signal,
        },
        IndicatorOutput {
            name: "histogram".to_string(),
            values: histogram,
        },
        IndicatorOutput {
            name: "fast_ema".to_string(),
            values: fast_ema,
        },
        IndicatorOutput {
            name: "slow_ema".to_string(),
            values: slow_ema,
        },
    ]
}
pub fn latest_macd_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> MacdResult {
    let last = match store.last_close() {
        Some(last) => last,
        None => return (None, None, None, None, None),
    };
    if store.len() == 1 {
        return (Some(0.0), Some(0.0), Some(0.0), Some(last), Some(last));
    }
    let previous_index = store.len() - 2;
    let previous_close = store.close[previous_index];
    let previous_fast = output_at(outputs, "fast_ema", previous_index).unwrap_or(previous_close);
    let previous_slow = output_at(outputs, "slow_ema", previous_index).unwrap_or(previous_close);
    let fast = ema_next(last, previous_fast, params.fast);
    let slow = ema_next(last, previous_slow, params.slow);
    let macd_line = fast - slow;
    let previous_signal = output_at(outputs, "signal", previous_index).unwrap_or(0.0);
    let signal = ema_next(macd_line, previous_signal, params.signal);
    let histogram = macd_line - signal;
    (
        Some(macd_line),
        Some(signal),
        Some(histogram),
        Some(fast),
        Some(slow),
    )
}
#[allow(dead_code)]
pub fn latest_ppo(bars: &[Bar], params: MacdParams) -> (Option<f64>, Option<f64>, Option<f64>) {
    let outputs = ppo(bars, params, &mut HashMap::new());
    let index = bars.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "ppo", index),
        output_at_vec(&outputs, "signal", index),
        output_at_vec(&outputs, "histogram", index),
    )
}
pub fn ppo_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let ppo_key = format!(
        "ppo:value:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let signal_key = format!(
        "ppo:signal:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let histogram_key = format!(
        "ppo:histogram:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    if let (Some(ppo), Some(signal), Some(histogram)) = (
        nodes.get(&ppo_key),
        nodes.get(&signal_key),
        nodes.get(&histogram_key),
    ) {
        return vec![
            IndicatorOutput {
                name: "ppo".to_string(),
                values: (**ppo).clone(),
            },
            IndicatorOutput {
                name: "signal".to_string(),
                values: (**signal).clone(),
            },
            IndicatorOutput {
                name: "histogram".to_string(),
                values: (**histogram).clone(),
            },
        ];
    }
    let macd_outputs = macd_store(store, params, nodes);
    let macd_line = macd_outputs
        .iter()
        .find(|output| output.name == "macd")
        .map(|output| output.values.clone())
        .unwrap_or_else(|| vec![f64::NAN; store.len()]);
    let slow_ema = macd_outputs
        .iter()
        .find(|output| output.name == "slow_ema")
        .map(|output| output.values.clone())
        .unwrap_or_else(|| vec![f64::NAN; store.len()]);
    let ppo: Series = macd_line
        .iter()
        .zip(slow_ema.iter())
        .map(|(macd, slow)| match (macd, slow) {
            (macd, slow) if !macd.is_nan() && !slow.is_nan() && *slow != 0.0 => {
                100.0 * *macd / *slow
            }
            (a, b) if !a.is_nan() && !b.is_nan() => 0.0,
            _ => f64::NAN,
        })
        .collect();
    let signal = ema_series(&ppo, params.signal);
    let histogram: Series = ppo
        .iter()
        .zip(signal.iter())
        .map(|(ppo, signal)| match (ppo, signal) {
            (ppo, signal) if !ppo.is_nan() && !signal.is_nan() => *ppo - *signal,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(ppo_key, Rc::new(ppo.clone()));
    nodes.insert(signal_key, Rc::new(signal.clone()));
    nodes.insert(histogram_key, Rc::new(histogram.clone()));
    vec![
        IndicatorOutput {
            name: "ppo".to_string(),
            values: ppo,
        },
        IndicatorOutput {
            name: "signal".to_string(),
            values: signal,
        },
        IndicatorOutput {
            name: "histogram".to_string(),
            values: histogram,
        },
    ]
}
pub fn latest_ppo_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let (macd_line, _, _, _, slow_ema) = latest_macd_store(store, params, outputs);
    let ppo = match (macd_line, slow_ema) {
        (Some(macd_line), Some(slow_ema)) if slow_ema != 0.0 => Some(100.0 * macd_line / slow_ema),
        (Some(_), Some(_)) => Some(0.0),
        _ => None,
    };
    let previous_signal = store
        .len()
        .checked_sub(2)
        .and_then(|index| output_at(outputs, "signal", index))
        .unwrap_or(ppo.unwrap_or(0.0));
    let signal = ppo.map(|ppo| ema_next(ppo, previous_signal, params.signal));
    let histogram = ppo.zip(signal).map(|(ppo, signal)| ppo - signal);
    (ppo, signal, histogram)
}
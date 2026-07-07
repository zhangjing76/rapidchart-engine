use crate::indicators::ema::{ema_close_store, ema_next, ema_series};
use crate::rc_into_owned;
use crate::IndicatorArena;
use crate::MacdParams;
use crate::NodeCache;
use crate::{output_at};
use crate::{CandleStore, Series};
use std::rc::Rc;

type MacdResult = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

pub fn macd_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
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
            crate::named_series("macd", (**macd).clone(),),
            crate::named_series("signal", (**signal).clone(),),
            crate::named_series("histogram", (**histogram).clone(),),
            crate::named_series("fast_ema", (**fast_ema).clone(),),
            crate::named_series("slow_ema", (**slow_ema).clone(),),
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
        crate::named_series("macd", macd,),
        crate::named_series("signal", signal,),
        crate::named_series("histogram", histogram,),
        crate::named_series("fast_ema", fast_ema,),
        crate::named_series("slow_ema", slow_ema,),
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
pub fn ppo_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
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
            crate::named_series("ppo", Rc::clone(ppo)),
            crate::named_series("signal", Rc::clone(signal)),
            crate::named_series("histogram", Rc::clone(histogram)),
        ];
    }
    let macd_outputs = macd_store(store, params, nodes);
    let macd_line = macd_outputs
        .iter()
        .find(|output| output.name == "macd")
        .map(|output| output.values.clone())
        .unwrap_or_else(|| Rc::new(vec![f64::NAN; store.len()]));
    let slow_ema = macd_outputs
        .iter()
        .find(|output| output.name == "slow_ema")
        .map(|output| output.values.clone())
        .unwrap_or_else(|| Rc::new(vec![f64::NAN; store.len()]));
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
        crate::named_series("ppo", ppo),
        crate::named_series("signal", signal),
        crate::named_series("histogram", histogram),
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

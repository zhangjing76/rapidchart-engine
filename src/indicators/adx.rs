use crate::indicators::atr::true_range_store;
use crate::value_at_slice;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

type AdxResult = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

const VALUE_SLOT: usize = 0;
const PLUS_DI_SLOT: usize = 1;
const MINUS_DI_SLOT: usize = 2;
const TR_AVG_SLOT: usize = 3;
const PLUS_DM_AVG_SLOT: usize = 4;
const MINUS_DM_AVG_SLOT: usize = 5;
const DX_SLOT: usize = 6;

pub fn directional_movement_store(store: &CandleStore, index: usize) -> (f64, f64) {
    if index == 0 {
        return (0.0, 0.0);
    }
    let up_move = store.high[index] - store.high[index - 1];
    let down_move = store.low[index - 1] - store.low[index];
    let plus_dm = if up_move > down_move && up_move > 0.0 {
        up_move
    } else {
        0.0
    };
    let minus_dm = if down_move > up_move && down_move > 0.0 {
        down_move
    } else {
        0.0
    };
    (plus_dm, minus_dm)
}
pub fn adx_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let key = format!("adx:ohlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return adx_outputs(
            Rc::clone(values),
            nodes
                .get(&format!("adx:plus_di:{period}"))
                .map(Rc::clone)
                .unwrap_or_else(|| Rc::new(Vec::new())),
            nodes
                .get(&format!("adx:minus_di:{period}"))
                .map(Rc::clone)
                .unwrap_or_else(|| Rc::new(Vec::new())),
            nodes
                .get(&format!("adx:tr_avg:{period}"))
                .map(Rc::clone)
                .unwrap_or_else(|| Rc::new(Vec::new())),
            nodes
                .get(&format!("adx:plus_dm_avg:{period}"))
                .map(Rc::clone)
                .unwrap_or_else(|| Rc::new(Vec::new())),
            nodes
                .get(&format!("adx:minus_dm_avg:{period}"))
                .map(Rc::clone)
                .unwrap_or_else(|| Rc::new(Vec::new())),
            nodes
                .get(&format!("adx:dx:{period}"))
                .map(Rc::clone)
                .unwrap_or_else(|| Rc::new(Vec::new())),
        );
    }
    let mut values = vec![f64::NAN; store.len()];
    let mut plus_di_values = vec![f64::NAN; store.len()];
    let mut minus_di_values = vec![f64::NAN; store.len()];
    let mut tr_avg_values = vec![f64::NAN; store.len()];
    let mut plus_dm_avg_values = vec![f64::NAN; store.len()];
    let mut minus_dm_avg_values = vec![f64::NAN; store.len()];
    let mut dx_values = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        return adx_outputs(
            Rc::new(values),
            Rc::new(plus_di_values),
            Rc::new(minus_di_values),
            Rc::new(tr_avg_values),
            Rc::new(plus_dm_avg_values),
            Rc::new(minus_dm_avg_values),
            Rc::new(dx_values),
        );
    }
    let mut tr_avg = (1..=period)
        .map(|index| true_range_store(store, index))
        .sum::<f64>()
        / period as f64;
    let mut plus_dm_avg = (1..=period)
        .map(|index| directional_movement_store(store, index).0)
        .sum::<f64>()
        / period as f64;
    let mut minus_dm_avg = (1..=period)
        .map(|index| directional_movement_store(store, index).1)
        .sum::<f64>()
        / period as f64;
    plus_di_values[period] = di_value(tr_avg, plus_dm_avg);
    minus_di_values[period] = di_value(tr_avg, minus_dm_avg);
    tr_avg_values[period] = tr_avg;
    plus_dm_avg_values[period] = plus_dm_avg;
    minus_dm_avg_values[period] = minus_dm_avg;
    dx_values[period] = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    for index in period + 1..store.len() {
        let (plus_dm, minus_dm) = directional_movement_store(store, index);
        tr_avg = (tr_avg * (period - 1) as f64 + true_range_store(store, index)) / period as f64;
        plus_dm_avg = (plus_dm_avg * (period - 1) as f64 + plus_dm) / period as f64;
        minus_dm_avg = (minus_dm_avg * (period - 1) as f64 + minus_dm) / period as f64;
        plus_di_values[index] = di_value(tr_avg, plus_dm_avg);
        minus_di_values[index] = di_value(tr_avg, minus_dm_avg);
        tr_avg_values[index] = tr_avg;
        plus_dm_avg_values[index] = plus_dm_avg;
        minus_dm_avg_values[index] = minus_dm_avg;
        dx_values[index] = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    }
    if store.len() > period * 2 {
        let mut adx = dx_values[period + 1..=period * 2]
            .iter()
            .copied()
            .sum::<f64>()
            / period as f64;
        values[period * 2] = adx;
        for index in period * 2 + 1..store.len() {
            adx = (adx * (period - 1) as f64 + {
                let __v = dx_values[index];
                if __v.is_nan() {
                    0.0
                } else {
                    __v
                }
            }) / period as f64;
            values[index] = adx;
        }
    }
    nodes.insert(key, Rc::new(values.clone()));
    nodes.insert(
        format!("adx:plus_di:{period}"),
        Rc::new(plus_di_values.clone()),
    );
    nodes.insert(
        format!("adx:minus_di:{period}"),
        Rc::new(minus_di_values.clone()),
    );
    nodes.insert(
        format!("adx:tr_avg:{period}"),
        Rc::new(tr_avg_values.clone()),
    );
    nodes.insert(
        format!("adx:plus_dm_avg:{period}"),
        Rc::new(plus_dm_avg_values.clone()),
    );
    nodes.insert(
        format!("adx:minus_dm_avg:{period}"),
        Rc::new(minus_dm_avg_values.clone()),
    );
    nodes.insert(format!("adx:dx:{period}"), Rc::new(dx_values.clone()));
    adx_outputs(
        Rc::new(values),
        Rc::new(plus_di_values),
        Rc::new(minus_di_values),
        Rc::new(tr_avg_values),
        Rc::new(plus_dm_avg_values),
        Rc::new(minus_dm_avg_values),
        Rc::new(dx_values),
    )
}
pub fn di_value(tr_avg: f64, dm_avg: f64) -> f64 {
    if tr_avg == 0.0 {
        0.0
    } else {
        100.0 * dm_avg / tr_avg
    }
}
pub fn dx_value(tr_avg: f64, plus_dm_avg: f64, minus_dm_avg: f64) -> f64 {
    let plus_di = di_value(tr_avg, plus_dm_avg);
    let minus_di = di_value(tr_avg, minus_dm_avg);
    let sum = plus_di + minus_di;
    if sum == 0.0 {
        0.0
    } else {
        100.0 * (plus_di - minus_di).abs() / sum
    }
}
pub fn adx_outputs(
    values: RcSeries,
    plus_di: RcSeries,
    minus_di: RcSeries,
    tr_avg: RcSeries,
    plus_dm_avg: RcSeries,
    minus_dm_avg: RcSeries,
    dx: RcSeries,
) -> Vec<crate::NamedSeries> {
    vec![
        crate::named_series("value", values),
        crate::named_series("plus_di", plus_di),
        crate::named_series("minus_di", minus_di),
        crate::named_series("tr_avg", tr_avg),
        crate::named_series("plus_dm_avg", plus_dm_avg),
        crate::named_series("minus_dm_avg", minus_dm_avg),
        crate::named_series("dx", dx),
    ]
}
pub fn latest_adx_store(store: &CandleStore, period: usize, outputs: &IndicatorArena) -> AdxResult {
    if period == 0 || store.len() <= period {
        return (None, None, None, None, None, None, None);
    }
    if store.len() <= period * 2 {
        let outputs = adx_store(store, period, &mut HashMap::new());
        let index = store.len() - 1;
        return (
            value_at_slice(outputs[VALUE_SLOT].values.as_slice(), index),
            value_at_slice(outputs[PLUS_DI_SLOT].values.as_slice(), index),
            value_at_slice(outputs[MINUS_DI_SLOT].values.as_slice(), index),
            value_at_slice(outputs[TR_AVG_SLOT].values.as_slice(), index),
            value_at_slice(outputs[PLUS_DM_AVG_SLOT].values.as_slice(), index),
            value_at_slice(outputs[MINUS_DM_AVG_SLOT].values.as_slice(), index),
            value_at_slice(outputs[DX_SLOT].values.as_slice(), index),
        );
    }
    let previous_index = store.len() - 2;
    let previous_outputs;
    let source_outputs = if outputs.value_at_slot(TR_AVG_SLOT, previous_index).is_some()
        && outputs
            .value_at_slot(PLUS_DM_AVG_SLOT, previous_index)
            .is_some()
        && outputs
            .value_at_slot(MINUS_DM_AVG_SLOT, previous_index)
            .is_some()
        && outputs.value_at_slot(DX_SLOT, previous_index).is_some()
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
            IndicatorArena::from_named_outputs(adx_store(&previous, period, &mut HashMap::new()));
        &previous_outputs
    };
    let tr_avg = (source_outputs
        .value_at_slot(TR_AVG_SLOT, previous_index)
        .unwrap_or(0.0)
        * (period - 1) as f64
        + true_range_store(store, store.len() - 1))
        / period as f64;
    let (plus_dm, minus_dm) = directional_movement_store(store, store.len() - 1);
    let plus_dm_avg = (source_outputs
        .value_at_slot(PLUS_DM_AVG_SLOT, previous_index)
        .unwrap_or(0.0)
        * (period - 1) as f64
        + plus_dm)
        / period as f64;
    let minus_dm_avg = (source_outputs
        .value_at_slot(MINUS_DM_AVG_SLOT, previous_index)
        .unwrap_or(0.0)
        * (period - 1) as f64
        + minus_dm)
        / period as f64;
    let plus_di = di_value(tr_avg, plus_dm_avg);
    let minus_di = di_value(tr_avg, minus_dm_avg);
    let dx = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    let value = if store.len() == period * 2 + 1 {
        let prior_dx_sum = (period + 1..=previous_index)
            .map(|index| source_outputs.value_at_slot(DX_SLOT, index).unwrap_or(0.0))
            .sum::<f64>();
        Some((prior_dx_sum + dx) / period as f64)
    } else {
        let previous_adx = source_outputs
            .value_at_slot(VALUE_SLOT, previous_index)
            .unwrap_or(0.0);
        Some((previous_adx * (period - 1) as f64 + dx) / period as f64)
    };
    (
        value,
        Some(plus_di),
        Some(minus_di),
        Some(tr_avg),
        Some(plus_dm_avg),
        Some(minus_dm_avg),
        Some(dx),
    )
}

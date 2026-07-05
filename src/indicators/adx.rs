use crate::indicators::atr::{true_range, true_range_store};
use crate::IndicatorArena;
use crate::IndicatorOutput;
use crate::NodeCache;
use crate::{output_at, output_at_vec};
use crate::{Bar, CandleStore, Series};
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

pub fn adx(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let key = format!("adx:ohlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return adx_outputs(
            (**values).clone(),
            nodes
                .get(&format!("adx:plus_di:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:minus_di:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:tr_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:plus_dm_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:minus_dm_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:dx:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
        );
    }
    let mut values = vec![f64::NAN; bars.len()];
    let mut plus_di_values = vec![f64::NAN; bars.len()];
    let mut minus_di_values = vec![f64::NAN; bars.len()];
    let mut tr_avg_values = vec![f64::NAN; bars.len()];
    let mut plus_dm_avg_values = vec![f64::NAN; bars.len()];
    let mut minus_dm_avg_values = vec![f64::NAN; bars.len()];
    let mut dx_values = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() <= period {
        return adx_outputs(
            values,
            plus_di_values,
            minus_di_values,
            tr_avg_values,
            plus_dm_avg_values,
            minus_dm_avg_values,
            dx_values,
        );
    }
    let mut tr_avg = (1..=period)
        .map(|index| true_range(bars, index))
        .sum::<f64>()
        / period as f64;
    let mut plus_dm_avg = (1..=period)
        .map(|index| directional_movement(bars, index).0)
        .sum::<f64>()
        / period as f64;
    let mut minus_dm_avg = (1..=period)
        .map(|index| directional_movement(bars, index).1)
        .sum::<f64>()
        / period as f64;
    plus_di_values[period] = di_value(tr_avg, plus_dm_avg);
    minus_di_values[period] = di_value(tr_avg, minus_dm_avg);
    tr_avg_values[period] = tr_avg;
    plus_dm_avg_values[period] = plus_dm_avg;
    minus_dm_avg_values[period] = minus_dm_avg;
    dx_values[period] = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    for index in period + 1..bars.len() {
        let (plus_dm, minus_dm) = directional_movement(bars, index);
        tr_avg = (tr_avg * (period - 1) as f64 + true_range(bars, index)) / period as f64;
        plus_dm_avg = (plus_dm_avg * (period - 1) as f64 + plus_dm) / period as f64;
        minus_dm_avg = (minus_dm_avg * (period - 1) as f64 + minus_dm) / period as f64;
        plus_di_values[index] = di_value(tr_avg, plus_dm_avg);
        minus_di_values[index] = di_value(tr_avg, minus_dm_avg);
        tr_avg_values[index] = tr_avg;
        plus_dm_avg_values[index] = plus_dm_avg;
        minus_dm_avg_values[index] = minus_dm_avg;
        dx_values[index] = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    }
    if bars.len() > period * 2 {
        let mut adx = dx_values[period + 1..=period * 2]
            .iter()
            .copied()
            .sum::<f64>()
            / period as f64;
        values[period * 2] = adx;
        for index in period * 2 + 1..bars.len() {
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
        values,
        plus_di_values,
        minus_di_values,
        tr_avg_values,
        plus_dm_avg_values,
        minus_dm_avg_values,
        dx_values,
    )
}
pub fn directional_movement(bars: &[Bar], index: usize) -> (f64, f64) {
    if index == 0 {
        return (0.0, 0.0);
    }
    let up_move = bars[index].high - bars[index - 1].high;
    let down_move = bars[index - 1].low - bars[index].low;
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
) -> Vec<IndicatorOutput> {
    let key = format!("adx:ohlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return adx_outputs(
            (**values).clone(),
            nodes
                .get(&format!("adx:plus_di:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:minus_di:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:tr_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:plus_dm_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:minus_dm_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:dx:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
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
            values,
            plus_di_values,
            minus_di_values,
            tr_avg_values,
            plus_dm_avg_values,
            minus_dm_avg_values,
            dx_values,
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
        values,
        plus_di_values,
        minus_di_values,
        tr_avg_values,
        plus_dm_avg_values,
        minus_dm_avg_values,
        dx_values,
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
    values: Series,
    plus_di: Series,
    minus_di: Series,
    tr_avg: Series,
    plus_dm_avg: Series,
    minus_dm_avg: Series,
    dx: Series,
) -> Vec<IndicatorOutput> {
    vec![
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "plus_di".to_string(),
            values: plus_di,
        },
        IndicatorOutput {
            name: "minus_di".to_string(),
            values: minus_di,
        },
        IndicatorOutput {
            name: "tr_avg".to_string(),
            values: tr_avg,
        },
        IndicatorOutput {
            name: "plus_dm_avg".to_string(),
            values: plus_dm_avg,
        },
        IndicatorOutput {
            name: "minus_dm_avg".to_string(),
            values: minus_dm_avg,
        },
        IndicatorOutput {
            name: "dx".to_string(),
            values: dx,
        },
    ]
}
#[allow(dead_code)]
pub fn latest_adx(bars: &[Bar], period: usize, outputs: &IndicatorArena) -> AdxResult {
    if period == 0 || bars.len() <= period {
        return (None, None, None, None, None, None, None);
    }
    if bars.len() <= period * 2 {
        let outputs = adx(bars, period, &mut HashMap::new());
        let index = bars.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "plus_di", index),
            output_at_vec(&outputs, "minus_di", index),
            output_at_vec(&outputs, "tr_avg", index),
            output_at_vec(&outputs, "plus_dm_avg", index),
            output_at_vec(&outputs, "minus_dm_avg", index),
            output_at_vec(&outputs, "dx", index),
        );
    }
    let previous_index = bars.len() - 2;
    let previous_outputs;
    let source_outputs = if output_at(outputs, "tr_avg", previous_index).is_some()
        && output_at(outputs, "plus_dm_avg", previous_index).is_some()
        && output_at(outputs, "minus_dm_avg", previous_index).is_some()
        && output_at(outputs, "dx", previous_index).is_some()
    {
        outputs
    } else {
        previous_outputs =
            IndicatorArena::from_outputs(adx(&bars[..bars.len() - 1], period, &mut HashMap::new()));
        &previous_outputs
    };
    let tr_avg = (output_at(source_outputs, "tr_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + true_range(bars, bars.len() - 1))
        / period as f64;
    let (plus_dm, minus_dm) = directional_movement(bars, bars.len() - 1);
    let plus_dm_avg = (output_at(source_outputs, "plus_dm_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + plus_dm)
        / period as f64;
    let minus_dm_avg = (output_at(source_outputs, "minus_dm_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + minus_dm)
        / period as f64;
    let plus_di = di_value(tr_avg, plus_dm_avg);
    let minus_di = di_value(tr_avg, minus_dm_avg);
    let dx = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    let value = if bars.len() == period * 2 + 1 {
        let prior_dx_sum = (period + 1..=previous_index)
            .map(|index| output_at(source_outputs, "dx", index).unwrap_or(0.0))
            .sum::<f64>();
        Some((prior_dx_sum + dx) / period as f64)
    } else {
        let previous_adx = output_at(source_outputs, "value", previous_index).unwrap_or(0.0);
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
pub fn latest_adx_store(store: &CandleStore, period: usize, outputs: &IndicatorArena) -> AdxResult {
    if period == 0 || store.len() <= period {
        return (None, None, None, None, None, None, None);
    }
    if store.len() <= period * 2 {
        let outputs = adx_store(store, period, &mut HashMap::new());
        let index = store.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "plus_di", index),
            output_at_vec(&outputs, "minus_di", index),
            output_at_vec(&outputs, "tr_avg", index),
            output_at_vec(&outputs, "plus_dm_avg", index),
            output_at_vec(&outputs, "minus_dm_avg", index),
            output_at_vec(&outputs, "dx", index),
        );
    }
    let previous_index = store.len() - 2;
    let previous_outputs;
    let source_outputs = if output_at(outputs, "tr_avg", previous_index).is_some()
        && output_at(outputs, "plus_dm_avg", previous_index).is_some()
        && output_at(outputs, "minus_dm_avg", previous_index).is_some()
        && output_at(outputs, "dx", previous_index).is_some()
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
            IndicatorArena::from_outputs(adx_store(&previous, period, &mut HashMap::new()));
        &previous_outputs
    };
    let tr_avg = (output_at(source_outputs, "tr_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + true_range_store(store, store.len() - 1))
        / period as f64;
    let (plus_dm, minus_dm) = directional_movement_store(store, store.len() - 1);
    let plus_dm_avg = (output_at(source_outputs, "plus_dm_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + plus_dm)
        / period as f64;
    let minus_dm_avg = (output_at(source_outputs, "minus_dm_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + minus_dm)
        / period as f64;
    let plus_di = di_value(tr_avg, plus_dm_avg);
    let minus_di = di_value(tr_avg, minus_dm_avg);
    let dx = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    let value = if store.len() == period * 2 + 1 {
        let prior_dx_sum = (period + 1..=previous_index)
            .map(|index| output_at(source_outputs, "dx", index).unwrap_or(0.0))
            .sum::<f64>();
        Some((prior_dx_sum + dx) / period as f64)
    } else {
        let previous_adx = output_at(source_outputs, "value", previous_index).unwrap_or(0.0);
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

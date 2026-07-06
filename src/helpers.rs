use js_sys::Object;
use js_sys::Reflect;
use wasm_bindgen::prelude::*;

use crate::bar::{Bar, CandleStore};
use crate::series::RcSeries;
use crate::types::IndicatorArena;

pub(crate) fn one_output(values: Vec<f64>) -> Vec<crate::types::IndicatorOutput> {
    vec![crate::types::IndicatorOutput {
        name: "value".to_string(),
        values,
    }]
}

pub(crate) fn rc_one_output(rc: RcSeries) -> Vec<crate::types::IndicatorOutput> {
    vec![crate::types::IndicatorOutput {
        name: "value".to_string(),
        values: crate::series::rc_into_owned(rc),
    }]
}

pub(crate) fn upsert_output(
    outputs: &mut IndicatorArena,
    name: &str,
    target_len: usize,
    value: Option<f64>,
) {
    let val = value.unwrap_or(f64::NAN);
    outputs.upsert_last(name, target_len, val);
}

pub(crate) fn output_at(outputs: &IndicatorArena, name: &str, index: usize) -> Option<f64> {
    outputs.value_at(name, index)
}

/// Same as output_at but for Vec<IndicatorOutput> used in internal compute functions.
pub(crate) fn output_at_vec(
    outputs: &[crate::types::IndicatorOutput],
    name: &str,
    index: usize,
) -> Option<f64> {
    outputs
        .iter()
        .find(|output| output.name == name)
        .and_then(|output| output.values.get(index))
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

#[allow(dead_code)]
pub(crate) fn upsert_bar(bars: &mut Vec<Bar>, bar: Bar) -> bool {
    match bars.last_mut() {
        Some(last) if last.time == bar.time => {
            *last = bar;
            true
        }
        Some(last) if last.time < bar.time => {
            bars.push(bar);
            true
        }
        None => {
            bars.push(bar);
            true
        }
        _ => false,
    }
}

pub(crate) fn upsert_candle_store(bars: &mut CandleStore, bar: Bar) -> bool {
    match bars.last_time() {
        Some(time) if time == bar.time => {
            bars.set_last(bar);
            true
        }
        Some(time) if time < bar.time => {
            bars.push(bar);
            true
        }
        None => {
            bars.push(bar);
            true
        }
        _ => false,
    }
}

pub(crate) fn js_set(target: &Object, key: &str, value: impl Into<JsValue>) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(key), &value.into())?;
    Ok(())
}
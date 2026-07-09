use js_sys::Object;
use js_sys::Reflect;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use crate::bar::{Bar, CandleStore};
use crate::series::{RcSeries, Series};
use crate::types::{IndicatorArena, NamedSeries};

pub(crate) trait IntoIndicatorArena {
    fn into_arena(self) -> IndicatorArena;
}

impl IntoIndicatorArena for RcSeries {
    fn into_arena(self) -> IndicatorArena {
        IndicatorArena::from_named_outputs(vec![NamedSeries {
            name: "value".to_string(),
            values: self,
        }])
    }
}

impl IntoIndicatorArena for Vec<NamedSeries> {
    fn into_arena(self) -> IndicatorArena {
        IndicatorArena::from_named_outputs(self)
    }
}

pub(crate) trait IntoRcSeries {
    fn into_rc_series(self) -> RcSeries;
}

impl IntoRcSeries for RcSeries {
    fn into_rc_series(self) -> RcSeries {
        self
    }
}

impl IntoRcSeries for Series {
    fn into_rc_series(self) -> RcSeries {
        Rc::new(self)
    }
}

pub(crate) fn named_series(name: impl Into<String>, values: impl IntoRcSeries) -> NamedSeries {
    NamedSeries {
        name: name.into(),
        values: values.into_rc_series(),
    }
}

/// Fast-path upsert for the incremental hot path. Resolves slot by name (1-7 slots typical).
#[inline]
pub(crate) fn upsert_output(
    outputs: &mut IndicatorArena,
    name: &str,
    target_len: usize,
    value: Option<f64>,
) {
    let val = value.unwrap_or(f64::NAN);
    outputs.upsert_last(name, target_len, val);
}

#[inline]
pub(crate) fn value_at_slice(values: &[f64], index: usize) -> Option<f64> {
    values
        .get(index)
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
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

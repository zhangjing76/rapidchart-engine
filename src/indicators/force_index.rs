use crate::{Bar, CandleStore, Series, RcSeries};
use crate::NodeCache;
use crate::{rc_into_owned, nan_to_none};
use std::rc::Rc;
use crate::indicators::ema::{ema_series};

pub fn force_index(bars: &[Bar], period: usize) -> Series {    let mut raw = vec![f64::NAN; bars.len()];    for index in 1..bars.len() {        raw[index] = (bars[index].close - bars[index - 1].close) * bars[index].volume;    }    ema_series(&raw, period)}
pub fn force_index_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {    let key = format!("force:close:volume:{period}");    if let Some(values) = nodes.get(&key) {        return (**values).clone();    }    let values = force_index(bars, period);    nodes.insert(key, Rc::new(values.clone()));    values}
pub fn force_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {    let key = format!("force:close:volume:{period}");    if let Some(values) = nodes.get(&key) {        return Rc::clone(values);    }    let mut raw = vec![f64::NAN; store.len()];    for index in 1..store.len() {        raw[index] = (store.close[index] - store.close[index - 1]) * store.volume[index];    }    let values = ema_series(&raw, period);    let rc = Rc::new(values); nodes.insert(key, Rc::clone(&rc));    rc}
pub fn latest_force_index(bars: &[Bar], period: usize) -> Option<f64> {    force_index(bars, period).last().copied().and_then(nan_to_none)}
pub fn latest_force_index_store(store: &CandleStore, period: usize) -> Option<f64> {    if store.len() < 2 {        return None;    }    let mut raw = vec![f64::NAN; store.len()];    for index in 1..store.len() {        raw[index] = (store.close[index] - store.close[index - 1]) * store.volume[index];    }    ema_series(&raw, period).last().copied().and_then(nan_to_none)}

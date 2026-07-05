use crate::{Bar, CandleStore, Series, RcSeries};
use crate::NodeCache;
use crate::{rc_into_owned, nan_to_none};
use std::rc::Rc;

pub fn roc(bars: &[Bar], period: usize) -> Series {    let mut out = vec![f64::NAN; bars.len()];    if period == 0 || bars.len() <= period {        return out;    }    for index in period..bars.len() {        let previous = bars[index - period].close;        out[index] = if previous == 0.0 { 0.0 } else { 100.0 * (bars[index].close / previous - 1.0) };    }    out}
pub fn roc_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {    let key = format!("roc:close:{period}");    if let Some(values) = nodes.get(&key) {        return (**values).clone();    }    let values = roc(bars, period);    nodes.insert(key, Rc::new(values.clone()));    values}
pub fn roc_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {    let key = format!("roc:close:{period}");    if let Some(values) = nodes.get(&key) {        return Rc::clone(values);    }    let mut out = vec![f64::NAN; store.len()];    if period == 0 || store.len() <= period {        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));        return rc;    }    for index in period..store.len() {        let previous = store.close[index - period];        out[index] = if previous == 0.0 { 0.0 } else { 100.0 * (store.close[index] / previous - 1.0) };    }    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));    rc}
pub fn latest_roc(bars: &[Bar], period: usize) -> Option<f64> {    roc(bars, period).last().copied().and_then(nan_to_none)}
pub fn latest_roc_store(store: &CandleStore, period: usize) -> Option<f64> {    if period == 0 || store.len() <= period {        return None;    }    let previous = store.close[store.len() - 1 - period];    Some(if previous == 0.0 {        0.0    } else {        100.0 * (store.close[store.len() - 1] / previous - 1.0)    })}

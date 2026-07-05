use crate::indicators::sma::{latest_sma_store, sma, sma_close, sma_close_store};
use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

#[allow(dead_code)]
pub fn dpo(bars: &[Bar], period: usize) -> Series {
    let sma_values = sma(bars, period);
    let shift = period / 2 + 1;
    let mut out = vec![f64::NAN; bars.len()];
    for (index, (out_val, &mean)) in out.iter_mut().zip(sma_values.iter()).enumerate() {
        if index < period.saturating_sub(1) || index < shift {
            continue;
        }
        if !mean.is_nan() {
            *out_val = bars[index - shift].close - mean;
        }
    }
    out
}
#[allow(dead_code)]
pub fn dpo_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("dpo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let sma_key = format!("sma:close:{period}");
    let sma_values = nodes
        .get(&sma_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| sma_close(bars, period, nodes));
    let shift = period / 2 + 1;
    let mut out = vec![f64::NAN; bars.len()];
    for (index, (out_val, &mean)) in out.iter_mut().zip(sma_values.iter()).enumerate() {
        if index < period.saturating_sub(1) || index < shift {
            continue;
        }
        if !mean.is_nan() {
            *out_val = bars[index - shift].close - mean;
        }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}
pub fn dpo_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("dpo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma_values = rc_into_owned(sma_close_store(store, period, nodes));
    let shift = period / 2 + 1;
    let mut out = vec![f64::NAN; store.len()];
    for (index, (out_val, &mean)) in out.iter_mut().zip(sma_values.iter()).enumerate() {
        if index < period.saturating_sub(1) || index < shift {
            continue;
        }
        if !mean.is_nan() {
            *out_val = store.close[index - shift] - mean;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
#[allow(dead_code)]
pub fn latest_dpo(bars: &[Bar], period: usize) -> Option<f64> {
    dpo(bars, period).last().copied().and_then(nan_to_none)
}
pub fn latest_dpo_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let shift = period / 2 + 1;
    let index = store.len() - 1;
    if index < shift || index < period.saturating_sub(1) {
        return None;
    }
    latest_sma_store(store, period).map(|mean| store.close[index - shift] - mean)
}

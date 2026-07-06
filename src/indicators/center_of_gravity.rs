use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Center of Gravity oscillator (Ehlers):
/// COG = -Sum(close[i] * (i+1), i=0..period-1) / Sum(close[i], i=0..period-1)
/// Higher weights on more recent bars; oscillates around zero.
pub fn center_of_gravity_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("cog:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..len {
        let window = &store.close[i + 1 - period..=i];
        let sum_weighted: f64 = window.iter().enumerate()
            .map(|(j, &c)| c * (j + 1) as f64).sum();
        let sum_close: f64 = window.iter().sum();
        if sum_close.abs() > 1e-10 {
            out[i] = -sum_weighted / sum_close;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn center_of_gravity_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("cog:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    for i in period - 1..len {
        let window = &bars[i + 1 - period..=i];
        let sum_weighted: f64 = window.iter().enumerate()
            .map(|(j, b)| b.close * (j + 1) as f64).sum();
        let sum_close: f64 = window.iter().map(|b| b.close).sum();
        if sum_close.abs() > 1e-10 {
            out[i] = -sum_weighted / sum_close;
        }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_center_of_gravity_store(store: &CandleStore, period: usize) -> Option<f64> {
    center_of_gravity_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

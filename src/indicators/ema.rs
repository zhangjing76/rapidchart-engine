use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn ema_close_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ema:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(ema_values(store.close.iter().copied(), period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn ema_values(values: impl IntoIterator<Item = f64>, period: usize) -> Series {
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut current = None::<f64>;
    let mut out = Vec::new();
    for value in values {
        let next = match current {
            Some(previous) => alpha * value + (1.0 - alpha) * previous,
            None => value,
        };
        current = Some(next);
        out.push(next);
    }
    out
}
pub fn ema_series(values: &[f64], period: usize) -> Series {
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut current = None::<f64>;
    let mut out = Vec::with_capacity(values.len());
    for &value in values {
        if value.is_nan() {
            out.push(f64::NAN);
        } else {
            let next = match current {
                Some(previous) => alpha * value + (1.0 - alpha) * previous,
                None => value,
            };
            current = Some(next);
            out.push(next);
        }
    }
    out
}
pub fn latest_ema_store(store: &CandleStore, period: usize, output: Option<&[f64]>) -> Option<f64> {
    let last = store.last_close()?;
    if period == 0 || store.len() == 1 {
        return Some(last);
    }
    let previous = output
        .and_then(|values| values.get(store.len() - 2))
        .copied()
        .and_then(nan_to_none)
        .unwrap_or(store.close[store.len() - 2]);
    Some(ema_next(last, previous, period))
}
pub fn ema_next(value: f64, previous: f64, period: usize) -> f64 {
    let alpha = 2.0 / (period as f64 + 1.0);
    alpha * value + (1.0 - alpha) * previous
}

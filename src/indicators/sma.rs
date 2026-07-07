use crate::NodeCache;
use crate::{CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn sma_close_values(values: &[f64], period: usize) -> Series {
    let mut out = Vec::with_capacity(values.len());
    let mut sum = 0.0;
    for (index, value) in values.iter().copied().enumerate() {
        sum += value;
        if index >= period {
            sum -= values[index - period];
        }
        out.push(if period > 0 && index + 1 >= period {
            sum / period as f64
        } else {
            f64::NAN
        });
    }
    out
}
pub fn latest_sma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    Some(store.close[store.len() - period..].iter().sum::<f64>() / period as f64)
}
pub fn sma_close_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("sma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(sma_close_values(&store.close, period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

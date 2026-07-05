use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn vwma(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let volume_sum = window.iter().map(|bar| bar.volume).sum::<f64>();
        if volume_sum == 0.0 {
            continue;
        }
        let weighted_sum = window.iter().map(|bar| bar.close * bar.volume).sum::<f64>();
        out[index] = weighted_sum / volume_sum;
    }
    out
}
pub fn vwma_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("vwma:close:volume:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = vwma(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}
pub fn vwma_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("vwma:close:volume:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for index in period - 1..store.len() {
        let start = index + 1 - period;
        let volume_sum = store.volume[start..=index].iter().sum::<f64>();
        if volume_sum == 0.0 {
            continue;
        }
        let weighted_sum = (start..=index)
            .map(|window_index| store.close[window_index] * store.volume[window_index])
            .sum::<f64>();
        out[index] = weighted_sum / volume_sum;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_vwma(bars: &[Bar], period: usize) -> Option<f64> {
    vwma(bars, period).last().copied().and_then(nan_to_none)
}
pub fn latest_vwma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let volume_sum = store.volume[start..].iter().sum::<f64>();
    if volume_sum == 0.0 {
        return None;
    }
    let weighted_sum = (start..store.len())
        .map(|index| store.close[index] * store.volume[index])
        .sum::<f64>();
    Some(weighted_sum / volume_sum)
}

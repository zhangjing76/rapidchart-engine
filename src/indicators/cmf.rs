use crate::indicators::adl::{money_flow_multiplier, money_flow_multiplier_parts};
use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn cmf(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let mfv_sum = window
            .iter()
            .map(|bar| money_flow_multiplier(bar) * bar.volume)
            .sum::<f64>();
        let volume_sum = window.iter().map(|bar| bar.volume).sum::<f64>();
        out[index] = if volume_sum != 0.0 {
            mfv_sum / volume_sum
        } else {
            f64::NAN
        };
    }
    out
}
pub fn cmf_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("cmf:hlcv:{period}");
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
        let mut mfv_sum = 0.0;
        let mut volume_sum = 0.0;
        for window_index in start..=index {
            mfv_sum += money_flow_multiplier_parts(
                store.high[window_index],
                store.low[window_index],
                store.close[window_index],
            ) * store.volume[window_index];
            volume_sum += store.volume[window_index];
        }
        out[index] = if volume_sum != 0.0 {
            mfv_sum / volume_sum
        } else {
            f64::NAN
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn cmf_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("cmf:hlcv:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = cmf(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}
pub fn latest_cmf(bars: &[Bar], period: usize) -> Option<f64> {
    cmf(bars, period).last().copied().and_then(nan_to_none)
}
pub fn latest_cmf_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let mut mfv_sum = 0.0;
    let mut volume_sum = 0.0;
    for index in start..store.len() {
        mfv_sum +=
            money_flow_multiplier_parts(store.high[index], store.low[index], store.close[index])
                * store.volume[index];
        volume_sum += store.volume[index];
    }
    (volume_sum != 0.0).then_some(mfv_sum / volume_sum)
}

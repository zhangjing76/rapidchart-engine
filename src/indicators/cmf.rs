use crate::indicators::adl::money_flow_multiplier_parts;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

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
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
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
        *item = if volume_sum != 0.0 {
            mfv_sum / volume_sum
        } else {
            f64::NAN
        };
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
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

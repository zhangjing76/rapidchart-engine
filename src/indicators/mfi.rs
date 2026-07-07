use crate::indicators::cci::typical_price_at;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn mfi_value(positive_flow: f64, negative_flow: f64) -> f64 {
    if negative_flow == 0.0 {
        100.0
    } else {
        let money_ratio = positive_flow / negative_flow;
        100.0 - 100.0 / (1.0 + money_ratio)
    }
}
pub fn mfi_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("mfi:hlcv:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period) {
        let mut positive_flow = 0.0;
        let mut negative_flow = 0.0;
        for current in index + 1 - period..=index {
            let previous = current - 1;
            let previous_tp = typical_price_at(store, previous);
            let current_tp = typical_price_at(store, current);
            let raw_flow = current_tp * store.volume[current];
            if current_tp > previous_tp {
                positive_flow += raw_flow;
            } else if current_tp < previous_tp {
                negative_flow += raw_flow;
            }
        }
        *item = mfi_value(positive_flow, negative_flow);
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_mfi_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;
    for current in store.len() - period..store.len() {
        let previous = current - 1;
        let previous_tp = typical_price_at(store, previous);
        let current_tp = typical_price_at(store, current);
        let raw_flow = current_tp * store.volume[current];
        if current_tp > previous_tp {
            positive_flow += raw_flow;
        } else if current_tp < previous_tp {
            negative_flow += raw_flow;
        }
    }
    Some(mfi_value(positive_flow, negative_flow))
}

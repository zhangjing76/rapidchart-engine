use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

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
    for (index, item) in out.iter_mut().enumerate().skip(period - 1) {
        let start = index + 1 - period;
        let volume_sum = store.volume[start..=index].iter().sum::<f64>();
        if volume_sum == 0.0 {
            continue;
        }
        let weighted_sum = (start..=index)
            .map(|window_index| store.close[window_index] * store.volume[window_index])
            .sum::<f64>();
        *item = weighted_sum / volume_sum;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
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

use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

pub fn stddev_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("stddev:close:{period}");
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
        let window = &store.close[index + 1 - period..=index];
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window
            .iter()
            .map(|close| {
                let diff = close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        *item = variance.sqrt();
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_stddev_store(store: &CandleStore, period: usize) -> Option<f64> {
    stddev_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}

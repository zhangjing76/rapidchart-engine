use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

/// Price Relative: ratio of current close to close N bars ago.
/// value[i] = close[i] / close[i - period]
/// This shows relative strength over the lookback period.
pub fn price_relative_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("price_relative:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len <= period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period..len {
        let prev = store.close[i - period];
        if prev != 0.0 {
            out[i] = store.close[i] / prev;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn price_relative_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("price_relative:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len <= period {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    for i in period..len {
        let prev = bars[i - period].close;
        if prev != 0.0 {
            out[i] = bars[i].close / prev;
        }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_price_relative_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    let i = store.len() - 1;
    let prev = store.close[i - period];
    if prev == 0.0 {
        None
    } else {
        Some(store.close[i] / prev)
    }
}

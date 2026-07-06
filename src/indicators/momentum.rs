use crate::nan_to_none;
use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

pub fn momentum(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if bars.len() <= period {
        return out;
    }
    for index in period..bars.len() {
        out[index] = bars[index].close - bars[index - period].close;
    }
    out
}
pub fn momentum_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("momentum:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if store.len() <= period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for (index, item) in out.iter_mut().enumerate().skip(period) {
        *item = store.close[index] - store.close[index - period];
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
#[allow(dead_code)]
pub fn latest_momentum(bars: &[Bar], period: usize) -> Option<f64> {
    momentum(bars, period).last().copied().and_then(nan_to_none)
}
pub fn latest_momentum_store(store: &CandleStore, period: usize) -> Option<f64> {
    momentum_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}
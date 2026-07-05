use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::rc::Rc;

pub fn sma(bars: &[Bar], period: usize) -> Series {
    let mut out = Vec::with_capacity(bars.len());
    let mut sum = 0.0;
    for (i, bar) in bars.iter().enumerate() {
        sum += bar.close;
        if i >= period {
            sum -= bars[i - period].close;
        }
        out.push(if i + 1 >= period {
            sum / period as f64
        } else {
            f64::NAN
        });
    }
    out
}
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
pub fn latest_sma(bars: &[Bar], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    Some(
        bars[bars.len() - period..]
            .iter()
            .map(|bar| bar.close)
            .sum::<f64>()
            / period as f64,
    )
}
pub fn latest_sma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    Some(store.close[store.len() - period..].iter().sum::<f64>() / period as f64)
}
pub fn sma_close(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("sma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = sma(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
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

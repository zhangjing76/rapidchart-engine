use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use crate::indicators::ema::{ema_close_store, ema_series};
use std::collections::HashMap;
use std::rc::Rc;

/// Elder Impulse System:
/// Combines EMA(13) direction and MACD Histogram direction into a signal.
/// +1 (green/bullish) = EMA rising AND MACD histogram rising
/// -1 (red/bearish) = EMA falling AND MACD histogram falling
///  0 (neutral) = mixed signals
pub fn elder_impulse_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("impulse:close:{period}");
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // EMA of close
    let ema = ema_close_store(store, period, nodes);
    // MACD histogram: EMA(12) - EMA(26) - Signal(EMA(9) of MACD line)
    let ema12 = ema_close_store(store, 12, nodes);
    let ema26 = ema_close_store(store, 26, nodes);
    let mut macd_line = vec![f64::NAN; len];
    for i in 0..len {
        if !ema12[i].is_nan() && !ema26[i].is_nan() {
            macd_line[i] = ema12[i] - ema26[i];
        }
    }
    let signal = ema_series(&macd_line, 9);
    let mut histogram = vec![f64::NAN; len];
    for i in 0..len {
        if !macd_line[i].is_nan() && !signal[i].is_nan() {
            histogram[i] = macd_line[i] - signal[i];
        }
    }
    for i in 1..len {
        if ema[i].is_nan() || ema[i - 1].is_nan() || histogram[i].is_nan() || histogram[i - 1].is_nan() {
            continue;
        }
        let ema_rising = ema[i] > ema[i - 1];
        let ema_falling = ema[i] < ema[i - 1];
        let hist_rising = histogram[i] > histogram[i - 1];
        let hist_falling = histogram[i] < histogram[i - 1];
        if ema_rising && hist_rising {
            out[i] = 1.0; // bullish
        } else if ema_falling && hist_falling {
            out[i] = -1.0; // bearish
        } else {
            out[i] = 0.0; // neutral
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_elder_impulse_store(store: &CandleStore, period: usize) -> Option<f64> {
    elder_impulse_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}
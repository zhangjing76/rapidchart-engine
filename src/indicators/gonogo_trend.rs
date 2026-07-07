use crate::indicators::ema::ema_series;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// GoNoGo Trend: A simplified trend-following signal.
/// Combines momentum (ROC) and moving average consensus into a signal.
/// +1 = strong bullish (close > EMA and momentum positive)
/// +0.5 = weak bullish
/// -0.5 = weak bearish
/// -1 = strong bearish (close < EMA and momentum negative)
/// 0 = neutral
pub fn gonogo_trend_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("gonogo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period + 1 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // EMA of close
    let ema = ema_series(&store.close, period);
    // Momentum: ROC over period/2
    let mom_period = (period / 2).max(1);
    for i in mom_period..len {
        if ema[i].is_nan() {
            continue;
        }
        let close = store.close[i];
        let prev_close = store.close[i - mom_period];
        let above_ema = close > ema[i];
        let below_ema = close < ema[i];
        let momentum_positive = prev_close > 0.0 && close > prev_close;
        let momentum_negative = prev_close > 0.0 && close < prev_close;
        if above_ema && momentum_positive {
            out[i] = 1.0; // strong bullish
        } else if above_ema {
            out[i] = 0.5; // weak bullish
        } else if below_ema && momentum_negative {
            out[i] = -1.0; // strong bearish
        } else if below_ema {
            out[i] = -0.5; // weak bearish
        } else {
            out[i] = 0.0;
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_gonogo_trend_store(store: &CandleStore, period: usize) -> Option<f64> {
    gonogo_trend_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

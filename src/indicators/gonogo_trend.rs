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

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "GONOGO_TREND",
        "GONOGO TREND",
        "Trend Analysis",
        "separate",
        14,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn close_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            vec![1.0; len],
        )
    }

    #[test]
    fn gonogo_trend_is_neutral_for_constant_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let values = gonogo_trend_store(&store, 3, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert_eq!(&*values[1..].to_vec(), &[0.0, 0.0, 0.0]);
        assert_eq!(latest_gonogo_trend_store(&store, 3), Some(0.0));
    }

    #[test]
    fn gonogo_trend_is_strong_bullish_when_price_rises_above_ema() {
        let store = close_store(&[10.0, 11.0, 12.0, 13.0]);
        let values = gonogo_trend_store(&store, 3, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert_eq!(&values[1..], &[1.0, 1.0, 1.0]);
        assert_eq!(latest_gonogo_trend_store(&store, 3), Some(1.0));
    }
}

use crate::indicators::ema::{ema_close_store, ema_series};
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Elder Impulse System:
/// Combines EMA(13) direction and MACD Histogram direction into a signal.
/// +1 (green/bullish) = EMA rising AND MACD histogram rising
/// -1 (red/bearish) = EMA falling AND MACD histogram falling
///  0 (neutral) = mixed signals
pub fn elder_impulse_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("impulse:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
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
        if ema[i].is_nan()
            || ema[i - 1].is_nan()
            || histogram[i].is_nan()
            || histogram[i - 1].is_nan()
        {
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
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
                "ELDER_IMPULSE",
                "ELDER IMPULSE SYSTEM",
                "Trend Analysis",
                "separate",
                13,
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
    fn elder_impulse_is_neutral_for_constant_prices() {
        let store = close_store(&[10.0, 10.0, 10.0, 10.0]);
        let values = elder_impulse_store(&store, 3, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert_eq!(&*values[1..].to_vec(), &[0.0, 0.0, 0.0]);
        assert_eq!(latest_elder_impulse_store(&store, 3), Some(0.0));
    }

    #[test]
    fn elder_impulse_turns_bullish_on_accelerating_prices() {
        let store = close_store(&[
            10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0,
            10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
            17.0, 18.0, 19.0, 20.0,
        ]);
        let values = elder_impulse_store(&store, 3, &mut HashMap::new());

        assert_eq!(&values[25..], &[1.0, 1.0, 1.0, 1.0, 1.0]);
        assert_eq!(latest_elder_impulse_store(&store, 3), Some(1.0));
    }
}

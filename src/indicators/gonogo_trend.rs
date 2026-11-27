use crate::indicators::kst::sma_from_series;
use crate::indicators::macd::macd_store;
use crate::indicators::parabolic_sar::parabolic_sar_store;
use crate::indicators::sma::sma_close_store;
use crate::MacdParams;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

const TREND_PERIOD: usize = 50;
const MOMENTUM_PERIOD: usize = 25;
const MOMENTUM_AVERAGE: usize = 5;
const SAR_STEP: f64 = 0.2;
const SAR_MAX_STEP: f64 = 0.2;
const MACD: MacdParams = MacdParams {
    fast: 12,
    slow: 26,
    signal: 9,
};

/// GoNoGo Trend: a score from 0 to 4, one point for each of
/// - the close above its 50-bar simple average,
/// - the close above a Parabolic SAR with step and maximum 0.2,
/// - the MACD (12, 26, 9) line and signal line both above zero,
/// - a 5-bar simple average of the 25-bar momentum (close minus the close 25 bars ago) above zero.
///
/// A condition whose input does not exist yet scores nothing, so the score is defined from bar 0.
pub fn gonogo_trend_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "gonogo_score:hlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let average = sma_close_store(store, TREND_PERIOD, nodes);
    let sar = parabolic_sar_store(store, SAR_STEP, SAR_MAX_STEP, nodes);
    let macd = macd_store(store, MACD, nodes);
    let momentum: Vec<f64> = (0..len)
        .map(|i| {
            if i >= MOMENTUM_PERIOD {
                store.close[i] - store.close[i - MOMENTUM_PERIOD]
            } else {
                f64::NAN
            }
        })
        .collect();
    let momentum_average = sma_from_series(&momentum, MOMENTUM_AVERAGE);
    let out: Vec<f64> = (0..len)
        .map(|i| {
            let close = store.close[i];
            // Comparisons with a missing (NaN) value are false, so they score nothing.
            [
                close > average[i],
                close > sar[0].values[i],
                macd[0].values[i] > 0.0 && macd[1].values[i] > 0.0,
                momentum_average[i] > 0.0,
            ]
            .iter()
            .filter(|&&met| met)
            .count() as f64
        })
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_gonogo_trend_store(store: &CandleStore) -> Option<f64> {
    gonogo_trend_store(store, &mut HashMap::new())
        .last()
        .copied()
        .and_then(crate::nan_to_none)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "GONOGO_TREND",
        name: "GONOGO TREND",
        category: "Trend Analysis",
        pane: "separate",
        params: Vec::new(),
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "separate", "#2563eb",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ohlc_store(closes: &[f64]) -> CandleStore {
        let len = closes.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            closes.to_vec(),
            closes.iter().map(|close| close + 1.0).collect(),
            closes.iter().map(|close| close - 1.0).collect(),
            closes.to_vec(),
            vec![1.0; len],
        )
    }

    #[test]
    fn gonogo_scores_nothing_before_any_input_exists() {
        let store = ohlc_store(&[10.0, 11.0, 12.0]);
        let values = gonogo_trend_store(&store, &mut HashMap::new());

        // The averages and MACD do not exist yet, and the SAR is only above the first bars.
        assert!(values.iter().all(|&score| (0.0..=4.0).contains(&score)));
        assert_eq!(values[0], 0.0);
    }

    #[test]
    fn gonogo_scores_four_on_a_long_steady_rise() {
        let closes: Vec<f64> = (0..120).map(|i| 100.0 + 2.0 * f64::from(i)).collect();
        let store = ohlc_store(&closes);
        let values = gonogo_trend_store(&store, &mut HashMap::new());

        assert_eq!(values[119], 4.0);
        assert_eq!(latest_gonogo_trend_store(&store), Some(4.0));
    }

    #[test]
    fn gonogo_scores_zero_on_a_long_steady_fall() {
        let closes: Vec<f64> = (0..120).map(|i| 400.0 - 2.0 * f64::from(i)).collect();
        let store = ohlc_store(&closes);
        let values = gonogo_trend_store(&store, &mut HashMap::new());

        assert_eq!(values[119], 0.0);
    }
}

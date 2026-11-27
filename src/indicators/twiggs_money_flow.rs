use crate::indicators::alligator::smma_values;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Twiggs Money Flow: Wilder-smoothed (alpha = 1/period) money flow volume over Wilder-smoothed
/// volume, both seeded with the simple average of their first `period` values from bar 1.
/// TMF = SMMA(ADV, period) / SMMA(volume, period)
/// where ADV = ((close - true_low) - (true_high - close)) / (true_high - true_low) * volume,
/// true_high = max(high, prev_close) and true_low = min(low, prev_close); ADV is 0 when the
/// true range is 0. The first value is at bar `period`.
pub fn twiggs_money_flow_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("tmf_wilder:hlcv:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut money_flow = vec![f64::NAN; len];
    let mut volume = vec![f64::NAN; len];
    for i in 1..len {
        let true_high = store.high[i].max(store.close[i - 1]);
        let true_low = store.low[i].min(store.close[i - 1]);
        let range = true_high - true_low;
        money_flow[i] = if range > 1e-10 {
            ((store.close[i] - true_low) - (true_high - store.close[i])) / range * store.volume[i]
        } else {
            0.0
        };
        volume[i] = store.volume[i];
    }
    let money_flow = smma_values(&money_flow, period);
    let volume = smma_values(&volume, period);
    let out: Vec<f64> = money_flow
        .iter()
        .zip(volume.iter())
        .map(|(&flow, &vol)| {
            if vol.abs() > 1e-10 {
                flow / vol
            } else {
                f64::NAN
            }
        })
        .collect();
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_twiggs_money_flow_store(store: &CandleStore, period: usize) -> Option<f64> {
    twiggs_money_flow_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "TWIGGS_MONEY_FLOW",
        "TWIGGS MONEY FLOW",
        "Volume",
        "separate",
        21,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            vec![1.0; len],
        )
    }

    #[test]
    fn twiggs_money_flow_is_one_when_close_stays_at_the_true_high() {
        let store = ohlcv_store(&[(2.0, 0.0, 2.0), (2.0, 0.0, 2.0), (2.0, 0.0, 2.0)]);
        let values = twiggs_money_flow_store(&store, 2, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!((values[2] - 1.0).abs() < 1e-12);
        assert_eq!(latest_twiggs_money_flow_store(&store, 2), Some(1.0));
    }

    #[test]
    fn twiggs_money_flow_uses_wilders_smoothing() {
        // Closes at the high (volume 1), the low (volume 3), then the high (volume 1).
        let store = CandleStore::from_raw_columns(
            vec![0, 1, 2, 3],
            vec![2.0, 2.0, 0.0, 2.0],
            vec![2.0, 2.0, 2.0, 2.0],
            vec![0.0, 0.0, 0.0, 0.0],
            vec![2.0, 2.0, 0.0, 2.0],
            vec![1.0, 1.0, 3.0, 1.0],
        );
        let values = twiggs_money_flow_store(&store, 2, &mut HashMap::new());

        // Money flow volumes from bar 1 are 1, -3, 1. Seeds: flow (1 - 3) / 2 = -1, volume 2.
        // Bar 3 moves each halfway to the new value: flow 0, volume 1.5.
        assert!((values[2] + 0.5).abs() < 1e-12);
        assert!(values[3].abs() < 1e-12);
    }
}

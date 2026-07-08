use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn money_flow_multiplier_parts(high: f64, low: f64, close: f64) -> f64 {
    let range = high - low;
    if range == 0.0 {
        0.0
    } else {
        ((close - low) - (high - close)) / range
    }
}
pub fn adl_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "adl:hlcv".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = Vec::with_capacity(store.len());
    let mut current = 0.0;
    for (((&high, &low), &close), &volume) in store
        .high
        .iter()
        .zip(store.low.iter())
        .zip(store.close.iter())
        .zip(store.volume.iter())
    {
        current += money_flow_multiplier_parts(high, low, close) * volume;
        out.push(current);
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_adl_store(store: &CandleStore, output: Option<&[f64]>) -> Option<f64> {
    let index = store.len().checked_sub(1)?;
    let previous = index
        .checked_sub(1)
        .and_then(|previous_index| {
            output
                .and_then(|values| values.get(previous_index))
                .copied()
                .and_then(nan_to_none)
        })
        .unwrap_or(0.0);
    Some(
        previous
            + money_flow_multiplier_parts(store.high[index], store.low[index], store.close[index])
                * store.volume[index],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(high, _, _, _)| *high).collect(),
            values.iter().map(|(_, low, _, _)| *low).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(_, _, _, volume)| *volume).collect(),
        )
    }

    fn assert_series_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else {
                assert!((actual - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn adl_is_the_manual_cumulative_money_flow() {
        let store = ohlcv_store(&[
            (4.0, 0.0, 3.0, 10.0),
            (5.0, 1.0, 2.0, 20.0),
            (6.0, 2.0, 5.0, 30.0),
        ]);
        let values = adl_store(&store, &mut HashMap::new());

        assert_series_close(&values, &[5.0, -5.0, 10.0]);
        assert_eq!(latest_adl_store(&store, Some(&values[..])), Some(10.0));
    }
}

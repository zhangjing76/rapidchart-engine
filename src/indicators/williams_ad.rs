use crate::nan_to_none;
use crate::CandleStore;
use crate::NodeCache;
use crate::RcSeries;
use std::rc::Rc;

pub fn williams_ad_step_parts(previous_close: f64, high: f64, low: f64, close: f64) -> f64 {
    if close > previous_close {
        close - previous_close.min(low)
    } else if close < previous_close {
        close - previous_close.max(high)
    } else {
        0.0
    }
}
pub fn williams_ad_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "wad:ohlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = Vec::with_capacity(store.len());
    let mut current = 0.0;
    for (index, (&high, (&low, &close))) in store
        .high
        .iter()
        .zip(store.low.iter().zip(store.close.iter()))
        .enumerate()
    {
        if index > 0 {
            current += williams_ad_step_parts(store.close[index - 1], high, low, close);
        }
        out.push(current);
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_williams_ad_store(store: &CandleStore, output: Option<&[f64]>) -> Option<f64> {
    let index = store.len().checked_sub(1)?;
    if index == 0 {
        return Some(0.0);
    }
    let previous = output
        .and_then(|values| values.get(index - 1))
        .copied()
        .and_then(nan_to_none)
        .unwrap_or(0.0);
    Some(
        previous
            + williams_ad_step_parts(
                store.close[index - 1],
                store.high[index],
                store.low[index],
                store.close[index],
            ),
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
    fn williams_ad_is_zero_for_constant_prices() {
        let store = ohlcv_store(&[(10.0, 10.0, 10.0), (10.0, 10.0, 10.0), (10.0, 10.0, 10.0)]);
        let values = williams_ad_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[0.0, 0.0, 0.0]);
        assert_eq!(
            latest_williams_ad_store(&store, Some(&values[..])),
            Some(0.0)
        );
    }

    #[test]
    fn williams_ad_accumulates_up_moves_and_subtracts_down_moves() {
        let store = ohlcv_store(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (12.0, 9.0, 10.0),
        ]);
        let values = williams_ad_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[0.0, 1.0, 2.0, 0.0]);
        assert_eq!(
            latest_williams_ad_store(&store, Some(&values[..])),
            Some(0.0)
        );
    }
}

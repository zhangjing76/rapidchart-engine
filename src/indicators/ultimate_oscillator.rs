use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn ultimate_oscillator_store(
    store: &CandleStore,
    short: usize,
    medium: usize,
    long: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("uo:{short}:{medium}:{long}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if short == 0 || medium == 0 || long == 0 || store.len() <= long {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let mut bp = vec![0.0; store.len()];
    let mut tr = vec![0.0; store.len()];
    for index in 1..store.len() {
        let previous_close = store.close[index - 1];
        let min_low = store.low[index].min(previous_close);
        let max_high = store.high[index].max(previous_close);
        bp[index] = store.close[index] - min_low;
        tr[index] = max_high - min_low;
    }
    for index in long..store.len() {
        let avg = |period: usize| {
            let start = index + 1 - period;
            let bp_sum = bp[start..=index].iter().sum::<f64>();
            let tr_sum = tr[start..=index].iter().sum::<f64>();
            if tr_sum == 0.0 {
                0.0
            } else {
                bp_sum / tr_sum
            }
        };
        out[index] = 100.0 * (4.0 * avg(short) + 2.0 * avg(medium) + avg(long)) / 7.0;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_ultimate_oscillator_store(
    store: &CandleStore,
    short: usize,
    medium: usize,
    long: usize,
) -> Option<f64> {
    if short == 0 || medium == 0 || long == 0 || store.len() <= long {
        return None;
    }
    let i = store.len() - 1;
    let avg = |period: usize| {
        let start = i + 1 - period;
        let mut bp_sum = 0.0;
        let mut tr_sum = 0.0;
        for j in start..=i {
            let prev_close = store.close[j - 1];
            let min_low = store.low[j].min(prev_close);
            let max_high = store.high[j].max(prev_close);
            bp_sum += store.close[j] - min_low;
            tr_sum += max_high - min_low;
        }
        if tr_sum == 0.0 {
            0.0
        } else {
            bp_sum / tr_sum
        }
    };
    Some(100.0 * (4.0 * avg(short) + 2.0 * avg(medium) + avg(long)) / 7.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[(f64, f64, f64)]) -> CandleStore {
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
    fn ultimate_oscillator_is_fifty_for_identical_bars() {
        let store = ohlc_store(&[
            (2.0, 0.0, 1.0),
            (2.0, 0.0, 1.0),
            (2.0, 0.0, 1.0),
            (2.0, 0.0, 1.0),
            (2.0, 0.0, 1.0),
        ]);
        let values = ultimate_oscillator_store(&store, 2, 3, 4, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, f64::NAN, f64::NAN, 50.0]);
        assert_eq!(latest_ultimate_oscillator_store(&store, 2, 3, 4), Some(50.0));
    }
}

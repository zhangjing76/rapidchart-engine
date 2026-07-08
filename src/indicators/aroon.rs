use crate::value_at_slice;
use crate::CandleStore;
use crate::NodeCache;
use std::collections::HashMap;
use std::rc::Rc;

pub fn aroon_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let key = format!("aroon:hl:{period}");
    if let Some(values) = nodes.get(&key) {
        return vec![
            crate::named_series("up", Rc::clone(values)),
            crate::named_series(
                "down",
                nodes
                    .get(&format!("aroon:down:{period}"))
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
            crate::named_series(
                "oscillator",
                nodes
                    .get(&format!("aroon:oscillator:{period}"))
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
        ];
    }
    let mut up = vec![f64::NAN; store.len()];
    let mut down = vec![f64::NAN; store.len()];
    let mut oscillator = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return vec![
            crate::named_series("up", up),
            crate::named_series("down", down),
            crate::named_series("oscillator", oscillator),
        ];
    }
    for index in period - 1..store.len() {
        let mut highest_index = 0;
        let mut highest = f64::NEG_INFINITY;
        let mut lowest_index = 0;
        let mut lowest = f64::INFINITY;
        for offset in 0..period {
            let window_index = index + 1 - period + offset;
            if store.high[window_index] > highest {
                highest = store.high[window_index];
                highest_index = offset;
            }
            if store.low[window_index] < lowest {
                lowest = store.low[window_index];
                lowest_index = offset;
            }
        }
        let periods_since_high = period - 1 - highest_index;
        let periods_since_low = period - 1 - lowest_index;
        let up_value = 100.0 * (period - periods_since_high) as f64 / period as f64;
        let down_value = 100.0 * (period - periods_since_low) as f64 / period as f64;
        up[index] = up_value;
        down[index] = down_value;
        oscillator[index] = up_value - down_value;
    }
    nodes.insert(key, Rc::new(up.clone()));
    nodes.insert(format!("aroon:down:{period}"), Rc::new(down.clone()));
    nodes.insert(
        format!("aroon:oscillator:{period}"),
        Rc::new(oscillator.clone()),
    );
    vec![
        crate::named_series("up", up),
        crate::named_series("down", down),
        crate::named_series("oscillator", oscillator),
    ]
}
pub fn latest_aroon_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let outputs = aroon_store(store, period, &mut HashMap::new());
    let index = store.len().saturating_sub(1);
    (
        value_at_slice(outputs[0].values.as_slice(), index),
        value_at_slice(outputs[1].values.as_slice(), index),
        value_at_slice(outputs[2].values.as_slice(), index),
    )
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
    fn aroon_is_the_manual_time_since_high_and_low() {
        let store = ohlc_store(&[
            (1.0, 1.0, 1.0),
            (2.0, 2.0, 2.0),
            (3.0, 3.0, 3.0),
            (4.0, 4.0, 4.0),
        ]);
        let outputs = aroon_store(&store, 3, &mut HashMap::new());

        assert_series_close(outputs[0].values.as_slice(), &[f64::NAN, f64::NAN, 100.0, 100.0]);
        assert_series_close(outputs[1].values.as_slice(), &[f64::NAN, f64::NAN, 33.333333333333336, 33.333333333333336]);
        assert_series_close(outputs[2].values.as_slice(), &[f64::NAN, f64::NAN, 66.66666666666666, 66.66666666666666]);
        assert_eq!(
            latest_aroon_store(&store, 3),
            (Some(100.0), Some(33.333333333333336), Some(66.66666666666666))
        );
    }
}

use crate::indicators::bollinger::bollinger_outputs;
use crate::CandleStore;
use crate::NodeCache;
use std::rc::Rc;

/// Highest high and lowest low of the `period` bars before `index`, not counting bar `index`.
fn previous_high_low(store: &CandleStore, period: usize, index: usize) -> (f64, f64) {
    let window = index - period..index;
    let high = store.high[window.clone()]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let low = store.low[window]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    (high, low)
}

/// Donchian Channel: the highest high and lowest low of the previous `period` bars, not counting
/// the current bar, so a breakout shows as price crossing the channel. The first value is at bar
/// `period`.
pub fn donchian_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let mut upper = vec![f64::NAN; store.len()];
    let mut middle = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        return bollinger_outputs(upper, middle, lower);
    }
    for index in period..store.len() {
        let (high, low) = previous_high_low(store, period, index);
        upper[index] = high;
        middle[index] = (high + low) / 2.0;
        lower[index] = low;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("donchian:{}:{}", output.name, period),
            Rc::clone(&output.values),
        );
    }
    outputs
}
pub fn latest_donchian_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() <= period {
        return (None, None, None);
    }
    let (high, low) = previous_high_low(store, period, store.len() - 1);
    (Some(high), Some((high + low) / 2.0), Some(low))
}
pub fn price_channel_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let mut upper = vec![f64::NAN; store.len()];
    let mut middle = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return bollinger_outputs(upper, middle, lower);
    }
    for index in period - 1..store.len() {
        let high = store.high[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let low = store.low[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        upper[index] = high;
        middle[index] = (high + low) / 2.0;
        lower[index] = low;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("price_channel:{}:{}", output.name, period),
            Rc::clone(&output.values),
        );
    }
    outputs
}
pub fn latest_price_channel_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let high = store.high[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let low = store.low[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    (Some(high), Some((high + low) / 2.0), Some(low))
}

pub(crate) fn price_channel_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "PRICE_CHANNEL",
        name: "PRICE CHANNEL",
        category: "Support/Resistance",
        pane: "overlay",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "period",
            label: "Period",
            default: 20.0,
            min: 1.0,
            step: "1",
        }],
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "line", "overlay", "#f59e0b"),
            crate::descriptors::output_descriptor("middle", "line", "overlay", "#64748b"),
            crate::descriptors::output_descriptor("lower", "line", "overlay", "#f59e0b"),
        ],
    }
}

pub(crate) fn donchian_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "DONCHIAN",
        name: "DONCHIAN",
        category: "Averages/Bands",
        pane: "overlay",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "period",
            label: "Period",
            default: 20.0,
            min: 1.0,
            step: "1",
        }],
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "line", "overlay", "#f59e0b"),
            crate::descriptors::output_descriptor("middle", "line", "overlay", "#64748b"),
            crate::descriptors::output_descriptor("lower", "line", "overlay", "#f59e0b"),
        ],
    }
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
        for (index, (actual, expected)) in actual.iter().zip(expected.iter()).enumerate() {
            if expected.is_nan() {
                assert!(
                    actual.is_nan(),
                    "index {index}: actual={actual:?} expected=NaN"
                );
            } else {
                assert!(
                    (actual - expected).abs() < 1e-9,
                    "index {index}: actual={actual:?} expected={expected:?}"
                );
            }
        }
    }

    #[test]
    fn donchian_is_the_high_low_channel_of_the_previous_bars() {
        let store = ohlc_store(&[
            (10.0, 6.0, 9.0),
            (12.0, 7.0, 10.0),
            (11.0, 8.0, 10.0),
            (15.0, 9.0, 14.0),
        ]);
        let outputs = donchian_store(&store, 2, &mut HashMap::new());
        let nan = f64::NAN;

        // Bar 2 uses bars 0-1 and bar 3 uses bars 1-2, not counting the bar itself.
        assert_series_close(outputs[0].values.as_slice(), &[nan, nan, 12.0, 12.0]);
        assert_series_close(outputs[1].values.as_slice(), &[nan, nan, 9.0, 9.5]);
        assert_series_close(outputs[2].values.as_slice(), &[nan, nan, 6.0, 7.0]);
        assert_eq!(
            latest_donchian_store(&store, 2),
            (Some(12.0), Some(9.5), Some(7.0))
        );
    }

    #[test]
    fn a_breakout_bar_closes_above_the_donchian_channel() {
        // The last bar's high (15) is above the previous two bars' high (12).
        let store = ohlc_store(&[(10.0, 6.0, 9.0), (12.0, 7.0, 10.0), (15.0, 9.0, 14.0)]);
        let outputs = donchian_store(&store, 2, &mut HashMap::new());
        assert!(store.close[2] > outputs[0].values[2]);
        assert_eq!(
            latest_donchian_store(&ohlc_store(&[(1.0, 0.0, 1.0)]), 1).0,
            None
        );
    }

    #[test]
    fn price_channel_matches_donchian_for_the_same_window() {
        let store = ohlc_store(&[(10.0, 6.0, 9.0), (12.0, 7.0, 10.0), (11.0, 8.0, 10.0)]);
        let outputs = price_channel_store(&store, 2, &mut HashMap::new());

        assert_series_close(outputs[0].values.as_slice(), &[f64::NAN, 12.0, 12.0]);
        assert_series_close(outputs[1].values.as_slice(), &[f64::NAN, 9.0, 9.5]);
        assert_series_close(outputs[2].values.as_slice(), &[f64::NAN, 6.0, 7.0]);
        assert_eq!(
            latest_price_channel_store(&store, 2),
            (Some(12.0), Some(9.5), Some(7.0))
        );
    }
}

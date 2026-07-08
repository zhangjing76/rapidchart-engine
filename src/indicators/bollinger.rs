use crate::indicators::sma::sma_close_store;
use crate::rc_into_owned;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::rc::Rc;

pub fn bollinger_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let mut upper = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    let middle = rc_into_owned(sma_close_store(store, period, nodes));
    if period == 0 {
        return bollinger_outputs(upper, middle, lower);
    }
    for index in period - 1..store.len() {
        let mean = middle[index];
        if mean.is_nan() {
            continue;
        };
        let variance = store.close[index + 1 - period..=index]
            .iter()
            .map(|close| {
                let diff = close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        let band = variance.sqrt() * multiplier;
        upper[index] = mean + band;
        lower[index] = mean - band;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("bb:{}:{}:{}", output.name, period, multiplier),
            Rc::clone(&output.values),
        );
    }
    outputs
}
pub fn bollinger_outputs(upper: Series, middle: Series, lower: Series) -> Vec<crate::NamedSeries> {
    vec![
        crate::named_series("upper", upper),
        crate::named_series("middle", middle),
        crate::named_series("lower", lower),
    ]
}
pub fn latest_bollinger_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let window = &store.close[store.len() - period..];
    let mean = window.iter().sum::<f64>() / period as f64;
    let variance = window
        .iter()
        .map(|close| {
            let diff = close - mean;
            diff * diff
        })
        .sum::<f64>()
        / period as f64;
    let band = variance.sqrt() * multiplier;
    (Some(mean + band), Some(mean), Some(mean - band))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "BB",
                name: "BOLLINGER",
                category: "Volatility",
                pane: "overlay",
                params: vec![
                    crate::descriptors::ParamDescriptor {
                        name: "period",
                        label: "Period",
                        default: 20.0,
                        min: 1.0,
                        step: "1",
                    },
                    crate::descriptors::ParamDescriptor {
                        name: "multiplier",
                        label: "Multiplier",
                        default: 2.0,
                        min: 1.0,
                        step: "0.1",
                    },
                ],
                outputs: vec![
                    crate::descriptors::output_descriptor("upper", "line", "overlay", "#9333ea"),
                    crate::descriptors::output_descriptor("middle", "line", "overlay", "#64748b"),
                    crate::descriptors::output_descriptor("lower", "line", "overlay", "#9333ea"),
                ],
            }
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
    fn bollinger_is_the_manual_mean_plus_stddev_band() {
        let store = close_store(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let outputs = bollinger_store(&store, 3, 2.0, &mut HashMap::new());
        let band = (2.0_f64 / 3.0).sqrt() * 2.0;

        assert_series_close(
            outputs[0].values.as_slice(),
            &[f64::NAN, f64::NAN, 2.0 + band, 3.0 + band, 4.0 + band],
        );
        assert_series_close(
            outputs[1].values.as_slice(),
            &[f64::NAN, f64::NAN, 2.0, 3.0, 4.0],
        );
        assert_series_close(
            outputs[2].values.as_slice(),
            &[f64::NAN, f64::NAN, 2.0 - band, 3.0 - band, 4.0 - band],
        );
        assert_eq!(
            latest_bollinger_store(&store, 3, 2.0),
            (Some(4.0 + band), Some(4.0), Some(4.0 - band))
        );
    }
}

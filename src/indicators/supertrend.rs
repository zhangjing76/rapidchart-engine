use crate::indicators::atr::{atr_store, latest_atr_store};
use crate::indicators::derived::{hl2_store, latest_hl2};
use crate::rc_into_owned;
use crate::value_at_slice;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, Series};
use std::collections::HashMap;
use std::rc::Rc;

const VALUE_SLOT: usize = 0;
const UPPER_BAND_SLOT: usize = 1;
const LOWER_BAND_SLOT: usize = 2;
const TREND_SLOT: usize = 3;

pub fn supertrend_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let atr = rc_into_owned(atr_store(store, period, nodes));
    let hl2 = hl2_store(store, nodes);
    let mut values = vec![f64::NAN; store.len()];
    let mut upper_band = vec![f64::NAN; store.len()];
    let mut lower_band = vec![f64::NAN; store.len()];
    let mut trend = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        return supertrend_outputs(values, upper_band, lower_band, trend);
    }
    for index in period..store.len() {
        let atr_value = atr[index];
        if atr_value.is_nan() {
            continue;
        };
        let hl2 = hl2[index];
        let basic_upper = hl2 + multiplier * atr_value;
        let basic_lower = hl2 - multiplier * atr_value;
        let previous_close = store.close[index - 1];
        let current_upper = if index == period {
            basic_upper
        } else {
            let previous_upper = {
                let __v = upper_band[index - 1];
                if __v.is_nan() {
                    basic_upper
                } else {
                    __v
                }
            };
            if basic_upper < previous_upper || previous_close > previous_upper {
                basic_upper
            } else {
                previous_upper
            }
        };
        let current_lower = if index == period {
            basic_lower
        } else {
            let previous_lower = {
                let __v = lower_band[index - 1];
                if __v.is_nan() {
                    basic_lower
                } else {
                    __v
                }
            };
            if basic_lower > previous_lower || previous_close < previous_lower {
                basic_lower
            } else {
                previous_lower
            }
        };
        upper_band[index] = current_upper;
        lower_band[index] = current_lower;
        let current_trend = if index == period {
            if store.close[index] >= hl2 {
                1.0
            } else {
                -1.0
            }
        } else {
            let previous_trend = {
                let __v = trend[index - 1];
                if __v.is_nan() {
                    1.0
                } else {
                    __v
                }
            };
            if previous_trend < 0.0 {
                if store.close[index] > current_upper {
                    1.0
                } else {
                    -1.0
                }
            } else if store.close[index] < current_lower {
                -1.0
            } else {
                1.0
            }
        };
        trend[index] = current_trend;
        values[index] = if current_trend < 0.0 {
            current_upper
        } else {
            current_lower
        };
    }
    nodes.insert(
        format!("supertrend:{period}:{multiplier}"),
        Rc::new(values.clone()),
    );
    supertrend_outputs(values, upper_band, lower_band, trend)
}
pub fn supertrend_outputs(
    values: Series,
    upper_band: Series,
    lower_band: Series,
    trend: Series,
) -> Vec<crate::NamedSeries> {
    vec![
        crate::named_series("value", values),
        crate::named_series("upper_band", upper_band),
        crate::named_series("lower_band", lower_band),
        crate::named_series("trend", trend),
    ]
}
pub fn latest_supertrend_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() <= period {
        return (None, None, None, None);
    }
    if store.len() == period + 1 {
        let outputs = supertrend_store(store, period, multiplier, &mut HashMap::new());
        let index = store.len() - 1;
        return (
            value_at_slice(outputs[VALUE_SLOT].values.as_slice(), index),
            value_at_slice(outputs[UPPER_BAND_SLOT].values.as_slice(), index),
            value_at_slice(outputs[LOWER_BAND_SLOT].values.as_slice(), index),
            value_at_slice(outputs[TREND_SLOT].values.as_slice(), index),
        );
    }
    let Some(atr_value) = latest_atr_store(store, period, None) else {
        return (None, None, None, None);
    };
    let index = store.len() - 1;
    let hl2 = latest_hl2(store).unwrap_or(f64::NAN);
    let basic_upper = hl2 + multiplier * atr_value;
    let basic_lower = hl2 - multiplier * atr_value;
    let previous_close = store.close[index - 1];
    let previous_upper = outputs
        .value_at_slot(UPPER_BAND_SLOT, index - 1)
        .unwrap_or(basic_upper);
    let previous_lower = outputs
        .value_at_slot(LOWER_BAND_SLOT, index - 1)
        .unwrap_or(basic_lower);
    let previous_trend = outputs.value_at_slot(TREND_SLOT, index - 1).unwrap_or(1.0);
    let upper = if basic_upper < previous_upper || previous_close > previous_upper {
        basic_upper
    } else {
        previous_upper
    };
    let lower = if basic_lower > previous_lower || previous_close < previous_lower {
        basic_lower
    } else {
        previous_lower
    };
    let trend = if previous_trend < 0.0 {
        if store.close[index] > upper {
            1.0
        } else {
            -1.0
        }
    } else if store.close[index] < lower {
        -1.0
    } else {
        1.0
    };
    let value = if trend < 0.0 { upper } else { lower };
    (Some(value), Some(upper), Some(lower), Some(trend))
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "SUPERTREND",
        name: "SUPERTREND",
        category: "Support/Resistance",
        pane: "overlay",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "period",
                label: "Period",
                default: 10.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "multiplier",
                label: "Multiplier",
                default: 3.0,
                min: 1.0,
                step: "0.1",
            },
        ],
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "overlay", "#0f766e",
        )],
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
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else {
                assert!((actual - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn supertrend_is_the_manual_trailing_band() {
        let store = ohlc_store(&[
            (1.0, 1.0, 1.0),
            (2.0, 2.0, 2.0),
            (3.0, 3.0, 3.0),
            (4.0, 4.0, 4.0),
            (5.0, 5.0, 5.0),
        ]);
        let outputs = supertrend_store(&store, 3, 2.0, &mut HashMap::new());
        let arena = crate::IndicatorArena::from_named_outputs(outputs.clone());

        assert_series_close(
            outputs[0].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 2.0, 3.0],
        );
        assert_series_close(
            outputs[1].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 6.0, 6.0],
        );
        assert_series_close(
            outputs[2].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 2.0, 3.0],
        );
        assert_series_close(
            outputs[3].values.as_slice(),
            &[f64::NAN, f64::NAN, f64::NAN, 1.0, 1.0],
        );
        assert_eq!(
            latest_supertrend_store(&store, 3, 2.0, &arena),
            (Some(3.0), Some(6.0), Some(3.0), Some(1.0))
        );
    }
}

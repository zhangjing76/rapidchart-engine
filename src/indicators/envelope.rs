use crate::indicators::sma::{latest_sma, latest_sma_store, sma_close, sma_close_store};
use crate::rc_into_owned;
use crate::IndicatorOutput;
use crate::NodeCache;
use crate::{Bar, CandleStore};
use std::rc::Rc;

#[allow(dead_code)]
pub fn envelope(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = sma_close(bars, period, nodes);
    let key_base = format!("envelope:{period}:{multiplier}");
    let upper_key = format!("envelope:upper:{period}:{multiplier}");
    let middle_key = format!("envelope:middle:{period}:{multiplier}");
    let lower_key = format!("envelope:lower:{period}:{multiplier}");
    if let (Some(upper), Some(middle), Some(lower)) = (
        nodes.get(&upper_key),
        nodes.get(&middle_key),
        nodes.get(&lower_key),
    ) {
        return vec![
            IndicatorOutput {
                name: "upper".to_string(),
                values: (**upper).clone(),
            },
            IndicatorOutput {
                name: "middle".to_string(),
                values: (**middle).clone(),
            },
            IndicatorOutput {
                name: "lower".to_string(),
                values: (**lower).clone(),
            },
        ];
    }
    let upper: Vec<_> = middle
        .iter()
        .map(|&value| {
            if value.is_nan() {
                f64::NAN
            } else {
                value * (1.0 + multiplier / 100.0)
            }
        })
        .collect();
    let lower: Vec<_> = middle
        .iter()
        .map(|&value| {
            if value.is_nan() {
                f64::NAN
            } else {
                value * (1.0 - multiplier / 100.0)
            }
        })
        .collect();
    nodes.insert(upper_key, Rc::new(upper.clone()));
    nodes.insert(middle_key, Rc::new(middle.clone()));
    nodes.insert(lower_key, Rc::new(lower.clone()));
    nodes.insert(key_base, Rc::new(middle.clone()));
    vec![
        IndicatorOutput {
            name: "upper".to_string(),
            values: upper,
        },
        IndicatorOutput {
            name: "middle".to_string(),
            values: middle,
        },
        IndicatorOutput {
            name: "lower".to_string(),
            values: lower,
        },
    ]
}
pub fn envelope_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = rc_into_owned(sma_close_store(store, period, nodes));
    let key_base = format!("envelope:{period}:{multiplier}");
    let upper_key = format!("envelope:upper:{period}:{multiplier}");
    let middle_key = format!("envelope:middle:{period}:{multiplier}");
    let lower_key = format!("envelope:lower:{period}:{multiplier}");
    if let (Some(upper), Some(middle), Some(lower)) = (
        nodes.get(&upper_key),
        nodes.get(&middle_key),
        nodes.get(&lower_key),
    ) {
        return vec![
            IndicatorOutput {
                name: "upper".to_string(),
                values: (**upper).clone(),
            },
            IndicatorOutput {
                name: "middle".to_string(),
                values: (**middle).clone(),
            },
            IndicatorOutput {
                name: "lower".to_string(),
                values: (**lower).clone(),
            },
        ];
    }
    let upper: Vec<_> = middle
        .iter()
        .map(|&value| {
            if value.is_nan() {
                f64::NAN
            } else {
                value * (1.0 + multiplier / 100.0)
            }
        })
        .collect();
    let lower: Vec<_> = middle
        .iter()
        .map(|&value| {
            if value.is_nan() {
                f64::NAN
            } else {
                value * (1.0 - multiplier / 100.0)
            }
        })
        .collect();
    nodes.insert(upper_key, Rc::new(upper.clone()));
    nodes.insert(middle_key, Rc::new(middle.clone()));
    nodes.insert(lower_key, Rc::new(lower.clone()));
    nodes.insert(key_base, Rc::new(middle.clone()));
    vec![
        IndicatorOutput {
            name: "upper".to_string(),
            values: upper,
        },
        IndicatorOutput {
            name: "middle".to_string(),
            values: middle,
        },
        IndicatorOutput {
            name: "lower".to_string(),
            values: lower,
        },
    ]
}
#[allow(dead_code)]
pub fn latest_envelope(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_sma(bars, period);
    let band = middle.map(|middle| middle * multiplier / 100.0);
    (
        middle.zip(band).map(|(middle, band)| middle + band),
        middle,
        middle.zip(band).map(|(middle, band)| middle - band),
    )
}
pub fn latest_envelope_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_sma_store(store, period);
    let band = middle.map(|middle| middle * multiplier / 100.0);
    (
        middle.zip(band).map(|(middle, band)| middle + band),
        middle,
        middle.zip(band).map(|(middle, band)| middle - band),
    )
}
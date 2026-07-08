use crate::indicators::sma::{latest_sma_store, sma_close_store};
use crate::rc_into_owned;
use crate::CandleStore;
use crate::NodeCache;
use std::rc::Rc;

pub fn envelope_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
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
            crate::named_series("upper", (**upper).clone()),
            crate::named_series("middle", (**middle).clone()),
            crate::named_series("lower", (**lower).clone()),
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
        crate::named_series("upper", upper),
        crate::named_series("middle", middle),
        crate::named_series("lower", lower),
    ]
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
    fn envelope_is_percent_bands_around_sma() {
        let store = close_store(&[1.0, 2.0, 3.0]);
        let outputs = envelope_store(&store, 2, 10.0, &mut HashMap::new());
        let middle = 1.5;

        assert_series_close(outputs[0].values.as_slice(), &[f64::NAN, 1.65, 2.75]);
        assert_series_close(outputs[1].values.as_slice(), &[f64::NAN, 1.5, 2.5]);
        assert_series_close(outputs[2].values.as_slice(), &[f64::NAN, 1.35, 2.25]);
        assert_eq!(
            latest_envelope_store(&store, 2, 10.0),
            (Some(2.75), Some(2.5), Some(2.25))
        );
        let _ = middle;
    }
}

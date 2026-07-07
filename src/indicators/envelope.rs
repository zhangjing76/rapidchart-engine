use crate::indicators::sma::{latest_sma_store, sma_close_store};
use crate::rc_into_owned;
use crate::NodeCache;
use crate::CandleStore;
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
            crate::named_series("upper", (**upper).clone(),),
            crate::named_series("middle", (**middle).clone(),),
            crate::named_series("lower", (**lower).clone(),),
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
        crate::named_series("upper", upper,),
        crate::named_series("middle", middle,),
        crate::named_series("lower", lower,),
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

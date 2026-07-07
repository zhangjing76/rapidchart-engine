use crate::NodeCache;
use crate::{CandleStore, IndicatorOutput};
use crate::indicators::sma::sma_close_store;
use crate::series::rc_into_owned;

/// Valuation Lines: A moving average with percentage offset lines above and below.
/// Outputs: upper (MA * (1 + pct/100)), middle (MA), lower (MA * (1 - pct/100)).

pub fn valuation_lines_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = rc_into_owned(sma_close_store(store, period, nodes));
    let pct = multiplier / 100.0;
    let upper: Vec<f64> = middle
        .iter()
        .map(|&m| if m.is_nan() { f64::NAN } else { m * (1.0 + pct) })
        .collect();
    let lower: Vec<f64> = middle
        .iter()
        .map(|&m| if m.is_nan() { f64::NAN } else { m * (1.0 - pct) })
        .collect();
    vec![
        IndicatorOutput { name: "upper".to_string(), values: upper },
        IndicatorOutput { name: "middle".to_string(), values: middle },
        IndicatorOutput { name: "lower".to_string(), values: lower },
    ]
}

pub fn latest_valuation_lines_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = crate::indicators::sma::latest_sma_store(store, period);
    match middle {
        Some(m) => {
            let pct = multiplier / 100.0;
            (Some(m * (1.0 + pct)), Some(m), Some(m * (1.0 - pct)))
        }
        None => (None, None, None),
    }
}
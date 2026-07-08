use crate::indicators::sma::sma_close_store;
use crate::series::rc_into_owned;
use crate::CandleStore;
use crate::NodeCache;

/// Valuation Lines: A moving average with percentage offset lines above and below.
/// Outputs: upper (MA * (1 + pct/100)), middle (MA), lower (MA * (1 - pct/100)).

pub fn valuation_lines_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let middle = rc_into_owned(sma_close_store(store, period, nodes));
    let pct = multiplier / 100.0;
    let upper: Vec<f64> = middle
        .iter()
        .map(|&m| {
            if m.is_nan() {
                f64::NAN
            } else {
                m * (1.0 + pct)
            }
        })
        .collect();
    let lower: Vec<f64> = middle
        .iter()
        .map(|&m| {
            if m.is_nan() {
                f64::NAN
            } else {
                m * (1.0 - pct)
            }
        })
        .collect();
    vec![
        crate::named_series("upper", upper),
        crate::named_series("middle", middle),
        crate::named_series("lower", lower),
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

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "VALUATION_LINES",
        name: "VALUATION LINES",
        category: "Statistical",
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
                label: "Offset %",
                default: 5.0,
                min: 0.1,
                step: "0.1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "line", "overlay", "#059669"),
            crate::descriptors::output_descriptor("middle", "line", "overlay", "#2563eb"),
            crate::descriptors::output_descriptor("lower", "line", "overlay", "#dc2626"),
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
    fn valuation_lines_apply_the_percentage_offset() {
        let store = close_store(&[10.0, 10.0, 10.0]);
        let values = valuation_lines_store(&store, 2, 10.0, &mut HashMap::new());

        assert_series_close(&values[0].values, &[f64::NAN, 11.0, 11.0]);
        assert_series_close(&values[1].values, &[f64::NAN, 10.0, 10.0]);
        assert_series_close(&values[2].values, &[f64::NAN, 9.0, 9.0]);
        assert_eq!(
            latest_valuation_lines_store(&store, 2, 10.0),
            (Some(11.0), Some(10.0), Some(9.0))
        );
    }

    #[test]
    fn valuation_lines_offset_a_rising_sma() {
        let store = close_store(&[10.0, 12.0, 14.0]);
        let values = valuation_lines_store(&store, 2, 10.0, &mut HashMap::new());

        assert_series_close(&values[0].values, &[f64::NAN, 12.1, 14.3]);
        assert_series_close(&values[1].values, &[f64::NAN, 11.0, 13.0]);
        assert_series_close(&values[2].values, &[f64::NAN, 9.9, 11.7]);
        let latest = latest_valuation_lines_store(&store, 2, 10.0);
        assert!((latest.0.unwrap() - 14.3).abs() < 1e-12);
        assert!((latest.1.unwrap() - 13.0).abs() < 1e-12);
        assert!((latest.2.unwrap() - 11.7).abs() < 1e-12);
    }
}

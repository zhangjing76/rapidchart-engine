use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Linear Regression Slope: the slope of the linear regression line over period bars.
pub fn linear_reg_slope_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("linreg_slope:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let n = period as f64;
    let sum_x = (0..period).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..period).map(|x| (x * x) as f64).sum::<f64>();
    let denom = n * sum_xx - sum_x * sum_x;
    if denom == 0.0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..len {
        let window = &store.close[i + 1 - period..=i];
        let sum_y: f64 = window.iter().sum();
        let sum_xy: f64 = window.iter().enumerate().map(|(x, c)| x as f64 * c).sum();
        out[i] = (n * sum_xy - sum_x * sum_y) / denom;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_linear_reg_slope_store(store: &CandleStore, period: usize) -> Option<f64> {
    linear_reg_slope_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
                "LINEAR_REG_SLOPE",
                "LINEAR REG SLOPE",
                "Statistical",
                "separate",
                14,
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
    fn linear_regression_slope_matches_the_line_slope() {
        let store = close_store(&[2.0, 4.0, 6.0]);
        let values = linear_reg_slope_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 2.0]);
        assert_eq!(latest_linear_reg_slope_store(&store, 3), Some(2.0));
    }
}

use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Linear Regression Forecast: projects the regression line value at the next bar (x = period).
/// For a window of `period` bars, compute the regression line and evaluate at x = period
/// (one step beyond the window end).
pub fn linear_reg_forecast_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("linreg_forecast:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let n = period as f64;
    let sum_x = (0..period).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..period).map(|x| (x * x) as f64).sum::<f64>();
    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator == 0.0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..store.len() {
        let window = &store.close[i + 1 - period..=i];
        let sum_y = window.iter().sum::<f64>();
        let sum_xy = window
            .iter()
            .enumerate()
            .map(|(offset, close)| offset as f64 * close)
            .sum::<f64>();
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;
        // Forecast: evaluate at x = period (one step beyond window end)
        out[i] = intercept + slope * period as f64;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_linear_reg_forecast_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let n = period as f64;
    let sum_x: f64 = (0..period).map(|x| x as f64).sum();
    let sum_xx: f64 = (0..period).map(|x| (x * x) as f64).sum();
    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator == 0.0 {
        return None;
    }
    let window = &store.close[store.len() - period..];
    let sum_y: f64 = window.iter().sum();
    let sum_xy: f64 = window.iter().enumerate().map(|(x, y)| x as f64 * y).sum();
    let slope = (n * sum_xy - sum_x * sum_y) / denominator;
    let intercept = (sum_y - slope * sum_x) / n;
    Some(intercept + slope * period as f64)
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
    fn linear_regression_forecast_extends_the_line() {
        let store = close_store(&[2.0, 4.0, 6.0]);
        let values = linear_reg_forecast_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 8.0]);
        assert_eq!(latest_linear_reg_forecast_store(&store, 3), Some(8.0));
    }
}

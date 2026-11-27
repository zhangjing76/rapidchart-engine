use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Least-squares line through `window`, with x = 0 at the oldest bar, evaluated at x = `at`.
fn line_value(window: &[f64], at: f64) -> Option<f64> {
    let n = window.len() as f64;
    let sum_x = (0..window.len()).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..window.len()).map(|x| (x * x) as f64).sum::<f64>();
    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator == 0.0 {
        return None;
    }
    let sum_y = window.iter().sum::<f64>();
    let sum_xy = window
        .iter()
        .enumerate()
        .map(|(x, y)| x as f64 * y)
        .sum::<f64>();
    let slope = (n * sum_xy - sum_x * sum_y) / denominator;
    let intercept = (sum_y - slope * sum_x) / n;
    Some(intercept + slope * at)
}

/// The regression line over the last `period` closes, evaluated `ahead` bars past the current bar.
fn regression_at_store(
    store: &CandleStore,
    period: usize,
    ahead: usize,
    key: String,
    nodes: &mut NodeCache,
) -> RcSeries {
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period > 0 && store.len() >= period {
        let at = (period - 1 + ahead) as f64;
        for i in period - 1..store.len() {
            if let Some(value) = line_value(&store.close[i + 1 - period..=i], at) {
                out[i] = value;
            }
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_regression_at(store: &CandleStore, period: usize, ahead: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    line_value(
        &store.close[store.len() - period..],
        (period - 1 + ahead) as f64,
    )
}

/// Linear Regression Forecast: the regression line over the last `period` closes, evaluated at
/// the current bar.
pub fn linear_reg_forecast_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    regression_at_store(
        store,
        period,
        0,
        format!("linreg_forecast_current:close:{period}"),
        nodes,
    )
}

pub fn latest_linear_reg_forecast_store(store: &CandleStore, period: usize) -> Option<f64> {
    latest_regression_at(store, period, 0)
}

/// Time Series Forecast: the regression line over the last `period` closes, projected one bar
/// past the current bar.
pub fn time_series_forecast_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    regression_at_store(store, period, 1, format!("tsf:close:{period}"), nodes)
}

pub fn latest_time_series_forecast_store(store: &CandleStore, period: usize) -> Option<f64> {
    latest_regression_at(store, period, 1)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "LINEAR_REG_FORECAST",
        "LINEAR REG FORECAST",
        "Statistical",
        "overlay",
        14,
    )
}

pub(crate) fn time_series_forecast_descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "TIME_SERIES_FORECAST",
        "TIME SERIES FORECAST",
        "Statistical",
        "overlay",
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
    fn linear_regression_forecast_is_the_line_at_the_current_bar() {
        let store = close_store(&[2.0, 4.0, 6.0, 7.0]);
        let values = linear_reg_forecast_store(&store, 3, &mut HashMap::new());

        // Bars 1-3 (4, 6, 7) fit y = 4.166.. + 1.5x, which is 7.166.. at the current bar.
        assert_series_close(&values, &[f64::NAN, f64::NAN, 6.0, 43.0 / 6.0]);
        assert_eq!(latest_linear_reg_forecast_store(&store, 3), Some(values[3]));
    }

    #[test]
    fn time_series_forecast_projects_one_bar_past_the_current_bar() {
        let store = close_store(&[2.0, 4.0, 6.0]);
        let values = time_series_forecast_store(&store, 3, &mut HashMap::new());

        assert_series_close(&values, &[f64::NAN, f64::NAN, 8.0]);
        assert_eq!(latest_time_series_forecast_store(&store, 3), Some(8.0));
    }
}

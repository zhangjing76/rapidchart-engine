use crate::nan_to_none;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
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
    linear_reg_forecast_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(nan_to_none)
}
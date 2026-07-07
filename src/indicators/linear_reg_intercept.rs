use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Linear Regression Intercept: the y-intercept of the regression line over period bars.
pub fn linear_reg_intercept_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("linreg_intercept:close:{period}");
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
        out[i] = intercept;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_linear_reg_intercept_store(store: &CandleStore, period: usize) -> Option<f64> {
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
    Some((sum_y - slope * sum_x) / n)
}
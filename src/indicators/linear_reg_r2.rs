use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Linear Regression R-Squared (Coefficient of Determination):
/// R² = 1 - (SS_res / SS_tot)
/// Measures how well the linear regression fits the price data over the period.
/// Range: 0 (no fit) to 1 (perfect linear fit).
pub fn linear_reg_r2_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("linreg_r2:close:{period}");
    if let Some(values) = nodes.get(&key) { return Rc::clone(values); }
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
        let sum_yy: f64 = window.iter().map(|y| y * y).sum();
        let sum_xy: f64 = window.iter().enumerate().map(|(x, y)| x as f64 * y).sum();
        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;
        // SS_tot = sum((y - mean_y)²)
        let mean_y = sum_y / n;
        let ss_tot = sum_yy - n * mean_y * mean_y;
        // SS_res = sum((y - predicted)²)
        let ss_res: f64 = window.iter().enumerate().map(|(x, &y)| {
            let predicted = intercept + slope * x as f64;
            (y - predicted).powi(2)
        }).sum();
        if ss_tot > 1e-10 {
            out[i] = 1.0 - ss_res / ss_tot;
        } else {
            out[i] = 1.0; // all values equal = perfect fit
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn linear_reg_r2_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("linreg_r2:close:{period}");
    if let Some(values) = nodes.get(&key) { return (**values).clone(); }
    let len = bars.len();
    let mut out = vec![f64::NAN; len];
    if period < 2 || len < period {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    let n = period as f64;
    let sum_x = (0..period).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..period).map(|x| (x * x) as f64).sum::<f64>();
    let denom = n * sum_xx - sum_x * sum_x;
    if denom == 0.0 {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    for i in period - 1..len {
        let window = &bars[i + 1 - period..=i];
        let sum_y: f64 = window.iter().map(|b| b.close).sum();
        let sum_yy: f64 = window.iter().map(|b| b.close * b.close).sum();
        let sum_xy: f64 = window.iter().enumerate().map(|(x, b)| x as f64 * b.close).sum();
        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;
        let mean_y = sum_y / n;
        let ss_tot = sum_yy - n * mean_y * mean_y;
        let ss_res: f64 = window.iter().enumerate().map(|(x, b)| {
            let predicted = intercept + slope * x as f64;
            (b.close - predicted).powi(2)
        }).sum();
        if ss_tot > 1e-10 { out[i] = 1.0 - ss_res / ss_tot; }
        else { out[i] = 1.0; }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_linear_reg_r2_store(store: &CandleStore, period: usize) -> Option<f64> {
    linear_reg_r2_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

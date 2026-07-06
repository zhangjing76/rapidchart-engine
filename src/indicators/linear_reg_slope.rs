use crate::NodeCache;
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

/// Linear Regression Slope: the slope of the linear regression line over period bars.
pub fn linear_reg_slope_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
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
        let sum_xy: f64 = window.iter().enumerate()
            .map(|(x, c)| x as f64 * c).sum();
        out[i] = (n * sum_xy - sum_x * sum_y) / denom;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn linear_reg_slope_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("linreg_slope:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
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
        let sum_xy: f64 = window.iter().enumerate()
            .map(|(x, b)| x as f64 * b.close).sum();
        out[i] = (n * sum_xy - sum_x * sum_y) / denom;
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

pub fn latest_linear_reg_slope_store(store: &CandleStore, period: usize) -> Option<f64> {
    linear_reg_slope_store(store, period, &mut HashMap::new())
        .last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
}

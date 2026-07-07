use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 || n == 3 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}

fn nearest_prime(value: f64) -> f64 {
    let rounded = value.round() as u64;
    if rounded < 2 { return 2.0; }
    if is_prime(rounded) { return rounded as f64; }
    let mut up = rounded + 1;
    let mut down = rounded.saturating_sub(1);
    loop {
        if is_prime(up) { return up as f64; }
        if down >= 2 && is_prime(down) { return down as f64; }
        up += 1;
        if down > 2 { down -= 1; } else { down = 2; }
    }
}

/// Prime Number Oscillator: close - nearest prime to close.
/// Positive when close is above the nearest prime, negative when below.
pub fn prime_number_oscillator_store(store: &CandleStore, _nodes: &mut NodeCache) -> RcSeries {
    let key = "pno:close".to_string();
    if let Some(values) = _nodes.get(&key) { return Rc::clone(values); }
    let len = store.len();
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let np = nearest_prime(store.close[i]);
        out.push(store.close[i] - np);
    }
    let rc = Rc::new(out);
    _nodes.insert(key, Rc::clone(&rc));
    rc
}


pub fn latest_prime_number_oscillator_store(store: &CandleStore) -> Option<f64> {
    if store.len() == 0 { return None; }
    let i = store.len() - 1;
    let np = nearest_prime(store.close[i]);
    Some(store.close[i] - np)
}
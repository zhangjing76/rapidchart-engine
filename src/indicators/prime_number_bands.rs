use crate::NodeCache;
use crate::{CandleStore};

/// Find the nearest prime number >= n (searching in integer space).
fn nearest_prime_up(value: f64) -> f64 {
    let mut n = value.ceil() as u64;
    if n < 2 {
        return 2.0;
    }
    loop {
        if is_prime(n) {
            return n as f64;
        }
        n += 1;
    }
}

/// Find the nearest prime number <= n (searching in integer space).
fn nearest_prime_down(value: f64) -> f64 {
    let mut n = value.floor() as u64;
    if n < 2 {
        return 2.0;
    }
    loop {
        if is_prime(n) {
            return n as f64;
        }
        if n == 2 {
            return 2.0;
        }
        n -= 1;
    }
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

/// Prime Number Bands: Upper band = nearest prime >= high, Lower band = nearest prime <= low.

pub fn prime_number_bands_store(store: &CandleStore, _nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    let len = store.len();
    let mut upper = Vec::with_capacity(len);
    let mut lower = Vec::with_capacity(len);
    for i in 0..len {
        upper.push(nearest_prime_up(store.high[i]));
        lower.push(nearest_prime_down(store.low[i]));
    }
    vec![
        crate::named_series("upper", upper),
        crate::named_series("lower", lower),
    ]
}

pub fn latest_prime_number_bands_store(store: &CandleStore) -> (Option<f64>, Option<f64>) {
    if store.len() == 0 {
        return (None, None);
    }
    let i = store.len() - 1;
    (
        Some(nearest_prime_up(store.high[i])),
        Some(nearest_prime_down(store.low[i])),
    )
}
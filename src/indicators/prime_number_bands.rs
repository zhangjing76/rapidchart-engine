use crate::CandleStore;
use crate::NodeCache;

/// Whether `value` is a whole prime number.
pub(crate) fn is_prime(value: f64) -> bool {
    if value < 2.0 || value.fract() != 0.0 || !value.is_finite() {
        return false;
    }
    let n = value as u64;
    if n < 4 {
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

/// Multiplies a positive price of 10 or less by 10 until it is above 10, so small prices still
/// land between whole-number primes. Returns the scaled value and the power of 10 used.
fn scale_up(mut value: f64) -> (f64, i32) {
    let mut power = 0;
    while value > 0.0 && value <= 10.0 {
        value *= 10.0;
        power += 1;
    }
    (value, power)
}

/// The first prime strictly above `high`, after scaling small prices (see [`scale_up`]).
pub(crate) fn prime_above(high: f64) -> f64 {
    if high.is_nan() || high <= 0.0 {
        return f64::NAN;
    }
    let (mut value, power) = scale_up(high);
    if is_prime(value) {
        value += 2.0;
    }
    value = value.ceil();
    if value % 2.0 == 0.0 {
        value += 1.0;
    }
    while !is_prime(value) {
        value += 2.0;
    }
    value / 10f64.powi(power)
}

/// The first prime strictly below `low`, after scaling small prices (see [`scale_up`]); 0 when
/// `low` is not positive.
pub(crate) fn prime_below(low: f64) -> f64 {
    if low.is_nan() {
        return f64::NAN;
    }
    if low <= 0.0 {
        return 0.0;
    }
    let (mut value, power) = scale_up(low);
    if is_prime(value) {
        value -= 2.0;
    }
    value = value.floor();
    if value % 2.0 == 0.0 {
        value -= 1.0;
    }
    // The scaled value is above 10, so 7 or a larger prime is always reached.
    while !is_prime(value) {
        value -= 2.0;
    }
    value / 10f64.powi(power)
}

/// Prime Number Bands: the upper band is the first prime above the bar's high and the lower band
/// the first prime below its low.
pub fn prime_number_bands_store(
    store: &CandleStore,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let upper: Vec<f64> = store.high.iter().map(|&high| prime_above(high)).collect();
    let lower: Vec<f64> = store.low.iter().map(|&low| prime_below(low)).collect();
    vec![
        crate::named_series("upper", upper),
        crate::named_series("lower", lower),
    ]
}

pub fn latest_prime_number_bands_store(store: &CandleStore) -> (Option<f64>, Option<f64>) {
    let Some(i) = store.len().checked_sub(1) else {
        return (None, None);
    };
    (
        crate::nan_to_none(prime_above(store.high[i])),
        crate::nan_to_none(prime_below(store.low[i])),
    )
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "PRIME_NUMBER_BANDS",
        name: "PRIME NUMBER BANDS",
        category: "Support/Resistance",
        pane: "overlay",
        params: Vec::new(),
        outputs: vec![
            crate::descriptors::output_descriptor("upper", "line", "overlay", "#059669"),
            crate::descriptors::output_descriptor("lower", "line", "overlay", "#dc2626"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[(f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            vec![1.0; len],
        )
    }

    #[test]
    fn prime_number_bands_move_strictly_past_the_price() {
        // A prime high or low moves on to the next prime; others go to the nearest one outside.
        let store = ohlc_store(&[(53.5, 47.0, 50.0), (59.0, 50.2, 55.0)]);
        let outputs = prime_number_bands_store(&store, &mut HashMap::new());

        assert_eq!(&*outputs[0].values, &[59.0, 61.0]);
        assert_eq!(&*outputs[1].values, &[43.0, 47.0]);
        assert_eq!(
            latest_prime_number_bands_store(&store),
            (Some(61.0), Some(47.0))
        );
    }

    #[test]
    fn prime_number_bands_scale_small_prices() {
        assert_eq!(prime_above(5.0), 5.3);
        assert_eq!(prime_below(5.0), 4.7);
        assert_eq!(prime_below(0.0), 0.0);
        assert!(prime_above(f64::NAN).is_nan());
    }
}

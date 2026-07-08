use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// QStick: SMA of (close - open) over period.
/// Positive = bullish candle bodies dominate, Negative = bearish.
pub fn qstick_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("qstick:oc:{period}");
    if let Some(v) = nodes.get(&key) {
        return Rc::clone(v);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for i in period - 1..len {
        let sum: f64 = (i + 1 - period..=i)
            .map(|j| store.close[j] - store.open[j])
            .sum();
        out[i] = sum / period as f64;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
pub fn latest_qstick_store(store: &CandleStore, period: usize) -> Option<f64> {
    qstick_store(store, period, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
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
    fn qstick_is_average_body_size() {
        let store = ohlc_store(&[(4.0, 2.0, 3.0), (5.0, 3.0, 4.0), (6.0, 4.0, 5.0)]);
        let values = qstick_store(&store, 2, &mut HashMap::new());

        assert!(values[0].is_nan());
        assert_eq!(values[1], 0.0);
        assert_eq!(values[2], 0.0);
        assert_eq!(latest_qstick_store(&store, 2), Some(0.0));
    }
}

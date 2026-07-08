use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

/// Projected Volume at Time:
/// Projects the expected volume at each bar position based on historical average.
/// Uses past `lookback` sessions (each of length `period`) to compute the average
/// volume at the same relative position within the session.
///
/// For bar i at position p within current session:
///   average of volume at position p across the previous `lookback` sessions.
pub fn projected_volume_at_time_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("pvat:v:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len == 0 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    // For each bar, compute the average volume at the same position
    // within previous complete session windows.
    for i in 0..len {
        let pos_in_session = i % period;
        let mut sum = 0.0;
        let mut count = 0u32;
        // Look back through previous sessions at the same position
        let mut idx = pos_in_session;
        while idx < i {
            sum += store.volume[idx];
            count += 1;
            idx += period;
        }
        if count > 0 {
            out[i] = sum / count as f64;
        } else {
            // No historical data at this position; use current volume
            out[i] = store.volume[i];
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_projected_volume_at_time_store(store: &CandleStore, period: usize) -> Option<f64> {
    if store.len() == 0 || period == 0 {
        return None;
    }
    let i = store.len() - 1;
    let pos_in_session = i % period;
    let mut sum = 0.0;
    let mut count = 0u32;
    let mut idx = pos_in_session;
    while idx < i {
        sum += store.volume[idx];
        count += 1;
        idx += period;
    }
    if count > 0 {
        Some(sum / count as f64)
    } else {
        Some(store.volume[i])
    }
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::period_descriptor(
        "PROJECTED_VOLUME_AT_TIME",
        "PROJECTED VOLUME AT TIME",
        "Volume",
        "separate",
        24,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn volume_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
        )
    }

    #[test]
    fn projected_volume_at_time_reuses_same_session_position() {
        let store = volume_store(&[10.0, 20.0, 30.0, 40.0, 50.0]);
        let values = projected_volume_at_time_store(&store, 2, &mut HashMap::new());

        assert_eq!(&*values, &[10.0, 20.0, 10.0, 20.0, 20.0]);
        assert_eq!(latest_projected_volume_at_time_store(&store, 2), Some(20.0));
    }
}

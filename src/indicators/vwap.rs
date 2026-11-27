use crate::indicators::cci::typical_price_parts;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

const CUMULATIVE_PV_SLOT: usize = 1;
const CUMULATIVE_VOLUME_SLOT: usize = 2;
const SECONDS_PER_DAY: u32 = 86_400;

/// VWAP restarts with each trading day. Bar times are seconds with the exchange's wall-clock
/// time read as UTC (as chart-rapid supplies them), so a new UTC calendar day is a new session.
fn starts_new_session(store: &CandleStore, index: usize) -> bool {
    index == 0 || store.time[index] / SECONDS_PER_DAY != store.time[index - 1] / SECONDS_PER_DAY
}

pub fn vwap_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    if let Some(values) = nodes.get("vwap:hlcv") {
        return vwap_outputs(
            Rc::clone(values),
            nodes
                .get("vwap:cumulative_pv")
                .map(Rc::clone)
                .unwrap_or_else(|| Rc::new(Vec::new())),
            nodes
                .get("vwap:cumulative_volume")
                .map(Rc::clone)
                .unwrap_or_else(|| Rc::new(Vec::new())),
        );
    }
    let mut values = Vec::with_capacity(store.len());
    let mut cumulative_pv_values = Vec::with_capacity(store.len());
    let mut cumulative_volume_values = Vec::with_capacity(store.len());
    let mut cumulative_pv = 0.0;
    let mut cumulative_volume = 0.0;
    for (index, (((&high, &low), &close), &volume)) in store
        .high
        .iter()
        .zip(store.low.iter())
        .zip(store.close.iter())
        .zip(store.volume.iter())
        .enumerate()
    {
        if starts_new_session(store, index) {
            cumulative_pv = 0.0;
            cumulative_volume = 0.0;
        }
        cumulative_pv += typical_price_parts(high, low, close) * volume;
        cumulative_volume += volume;
        values.push(if cumulative_volume > 0.0 {
            cumulative_pv / cumulative_volume
        } else {
            f64::NAN
        });
        cumulative_pv_values.push(cumulative_pv);
        cumulative_volume_values.push(cumulative_volume);
    }
    nodes.insert("vwap:hlcv".to_string(), Rc::new(values.clone()));
    nodes.insert(
        "vwap:cumulative_pv".to_string(),
        Rc::new(cumulative_pv_values.clone()),
    );
    nodes.insert(
        "vwap:cumulative_volume".to_string(),
        Rc::new(cumulative_volume_values.clone()),
    );
    vwap_outputs(
        Rc::new(values),
        Rc::new(cumulative_pv_values),
        Rc::new(cumulative_volume_values),
    )
}
pub fn vwap_outputs(
    values: RcSeries,
    cumulative_pv: RcSeries,
    cumulative_volume: RcSeries,
) -> Vec<crate::NamedSeries> {
    vec![
        crate::named_series("value", values),
        crate::named_series("cumulative_pv", cumulative_pv),
        crate::named_series("cumulative_volume", cumulative_volume),
    ]
}
pub fn latest_vwap_store(
    store: &CandleStore,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let Some(index) = store.len().checked_sub(1) else {
        return (None, None, None);
    };
    // The running totals carry over from the previous bar unless this bar starts a new day.
    let previous_index = index
        .checked_sub(1)
        .filter(|_| !starts_new_session(store, index));
    let previous_pv = previous_index
        .and_then(|previous_index| outputs.value_at_slot(CUMULATIVE_PV_SLOT, previous_index))
        .unwrap_or(0.0);
    let previous_volume = previous_index
        .and_then(|previous_index| outputs.value_at_slot(CUMULATIVE_VOLUME_SLOT, previous_index))
        .unwrap_or(0.0);
    let cumulative_pv = previous_pv
        + typical_price_parts(store.high[index], store.low[index], store.close[index])
            * store.volume[index];
    let cumulative_volume = previous_volume + store.volume[index];
    (
        (cumulative_volume > 0.0).then_some(cumulative_pv / cumulative_volume),
        Some(cumulative_pv),
        Some(cumulative_volume),
    )
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "VWAP",
        name: "VWAP",
        category: "Volume",
        pane: "overlay",
        params: Vec::new(),
        outputs: vec![crate::descriptors::output_descriptor(
            "value", "line", "overlay", "#0f766e",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64, f64)]) -> CandleStore {
        let times: Vec<u32> = (0..values.len() as u32).collect();
        ohlcv_store_at(&times, values)
    }

    fn ohlcv_store_at(times: &[u32], values: &[(f64, f64, f64, f64)]) -> CandleStore {
        CandleStore::from_raw_columns(
            times.to_vec(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(high, _, _, _)| *high).collect(),
            values.iter().map(|(_, low, _, _)| *low).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(_, _, _, volume)| *volume).collect(),
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
    fn vwap_tracks_cumulative_typical_price() {
        let store = ohlcv_store(&[(12.0, 6.0, 9.0, 2.0), (15.0, 9.0, 12.0, 3.0)]);
        let outputs = vwap_store(&store, &mut HashMap::new());
        let arena = crate::IndicatorArena::from_named_outputs(outputs.clone());

        assert_series_close(outputs[0].values.as_slice(), &[9.0, 10.8]);
        assert_series_close(outputs[1].values.as_slice(), &[18.0, 54.0]);
        assert_series_close(outputs[2].values.as_slice(), &[2.0, 5.0]);
        assert_eq!(
            latest_vwap_store(&store, &arena),
            (Some(10.8), Some(54.0), Some(5.0))
        );
    }

    #[test]
    fn vwap_restarts_each_day() {
        // Two bars late on one day, then two on the next (10:00 and 10:02 UTC).
        let day = SECONDS_PER_DAY;
        let times = [day - 120, day - 60, day + 36_000, day + 36_120];
        let store = ohlcv_store_at(
            &times,
            &[
                (12.0, 6.0, 9.0, 2.0),
                (15.0, 9.0, 12.0, 3.0),
                (21.0, 15.0, 18.0, 1.0),
                (24.0, 18.0, 21.0, 1.0),
            ],
        );
        let outputs = vwap_store(&store, &mut HashMap::new());

        // Day one: 9 then (18 + 36) / 5. Day two starts again: 18 then (18 + 21) / 2.
        assert_series_close(outputs[0].values.as_slice(), &[9.0, 10.8, 18.0, 19.5]);
        assert_series_close(outputs[1].values.as_slice(), &[18.0, 54.0, 18.0, 39.0]);
        assert_series_close(outputs[2].values.as_slice(), &[2.0, 5.0, 1.0, 2.0]);
    }

    #[test]
    fn latest_vwap_restarts_on_the_first_bar_of_a_day() {
        let day = SECONDS_PER_DAY;
        let values = [
            (12.0, 6.0, 9.0, 2.0),
            (15.0, 9.0, 12.0, 3.0),
            (21.0, 15.0, 18.0, 1.0),
        ];
        let full = ohlcv_store_at(&[day - 120, day - 60, day + 36_000], &values);
        let earlier = ohlcv_store_at(&[day - 120, day - 60], &values[..2]);
        let arena =
            crate::IndicatorArena::from_named_outputs(vwap_store(&earlier, &mut HashMap::new()));
        // The arena holds the first two bars; the new bar opens the next day.
        assert_eq!(
            latest_vwap_store(&full, &arena),
            (Some(18.0), Some(18.0), Some(1.0))
        );
        assert_eq!(vwap_store(&full, &mut HashMap::new())[0].values[2], 18.0);
    }
}

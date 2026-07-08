use crate::indicators::cci::typical_price_parts;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

const CUMULATIVE_PV_SLOT: usize = 1;
const CUMULATIVE_VOLUME_SLOT: usize = 2;

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
    for (((&high, &low), &close), &volume) in store
        .high
        .iter()
        .zip(store.low.iter())
        .zip(store.close.iter())
        .zip(store.volume.iter())
    {
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
    let previous_index = index.checked_sub(1);
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
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
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
}

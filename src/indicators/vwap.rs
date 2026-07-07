use crate::indicators::cci::typical_price_parts;
use crate::output_at;
use crate::IndicatorArena;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::rc::Rc;

pub fn vwap_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    if let Some(values) = nodes.get("vwap:hlcv") {
        return vwap_outputs(
            Rc::clone(values),
            nodes.get("vwap:cumulative_pv").map(Rc::clone).unwrap_or_else(|| Rc::new(Vec::new())),
            nodes.get("vwap:cumulative_volume").map(Rc::clone).unwrap_or_else(|| Rc::new(Vec::new())),
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
    vwap_outputs(Rc::new(values), Rc::new(cumulative_pv_values), Rc::new(cumulative_volume_values))
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
        .and_then(|previous_index| output_at(outputs, "cumulative_pv", previous_index))
        .unwrap_or(0.0);
    let previous_volume = previous_index
        .and_then(|previous_index| output_at(outputs, "cumulative_volume", previous_index))
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

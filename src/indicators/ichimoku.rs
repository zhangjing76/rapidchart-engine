use crate::value_at_slice;
use crate::CandleStore;
use crate::NodeCache;
use std::collections::HashMap;
use std::rc::Rc;

type IchimokuResult = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

pub fn ichimoku_store(
    store: &CandleStore,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let tenkan_key = format!("ichimoku:tenkan:{tenkan_period}");
    let kijun_key = format!("ichimoku:kijun:{kijun_period}");
    let senkou_a_key = format!("ichimoku:senkou_a:{tenkan_period}:{kijun_period}");
    let senkou_b_key = format!("ichimoku:senkou_b:{senkou_b_period}");
    if let Some(values) = nodes.get(&tenkan_key) {
        return vec![
            crate::named_series("tenkan", Rc::clone(values)),
            crate::named_series(
                "kijun",
                nodes
                    .get(&kijun_key)
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
            crate::named_series(
                "senkou_a",
                nodes
                    .get(&senkou_a_key)
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
            crate::named_series(
                "senkou_b",
                nodes
                    .get(&senkou_b_key)
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
            crate::named_series(
                "chikou",
                nodes
                    .get("ichimoku:chikou")
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
        ];
    }
    let mut tenkan = vec![f64::NAN; store.len()];
    let mut kijun = vec![f64::NAN; store.len()];
    let mut senkou_a = vec![f64::NAN; store.len()];
    let mut senkou_b = vec![f64::NAN; store.len()];
    let chikou = store.close.to_vec();
    for (index, (((tenkan_val, kijun_val), senkou_a_val), senkou_b_val)) in tenkan
        .iter_mut()
        .zip(kijun.iter_mut())
        .zip(senkou_a.iter_mut())
        .zip(senkou_b.iter_mut())
        .enumerate()
    {
        if index + 1 >= tenkan_period {
            *tenkan_val = midpoint_store(store, index + 1 - tenkan_period, index);
        }
        if index + 1 >= kijun_period {
            *kijun_val = midpoint_store(store, index + 1 - kijun_period, index);
        }
        let tenkan_value = *tenkan_val;
        let kijun_value = *kijun_val;
        if !tenkan_value.is_nan() && !kijun_value.is_nan() {
            *senkou_a_val = (tenkan_value + kijun_value) / 2.0;
        }
        if index + 1 >= senkou_b_period {
            *senkou_b_val = midpoint_store(store, index + 1 - senkou_b_period, index);
        }
    }
    nodes.insert(tenkan_key, Rc::new(tenkan.clone()));
    nodes.insert(kijun_key, Rc::new(kijun.clone()));
    nodes.insert(senkou_a_key, Rc::new(senkou_a.clone()));
    nodes.insert(senkou_b_key, Rc::new(senkou_b.clone()));
    nodes.insert("ichimoku:chikou".to_string(), Rc::new(chikou.clone()));
    vec![
        crate::named_series("tenkan", tenkan),
        crate::named_series("kijun", kijun),
        crate::named_series("senkou_a", senkou_a),
        crate::named_series("senkou_b", senkou_b),
        crate::named_series("chikou", chikou),
    ]
}
pub fn latest_ichimoku_store(
    store: &CandleStore,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
) -> IchimokuResult {
    let outputs = ichimoku_store(
        store,
        tenkan_period,
        kijun_period,
        senkou_b_period,
        &mut HashMap::new(),
    );
    let index = store.len().saturating_sub(1);
    (
        value_at_slice(outputs[0].values.as_slice(), index),
        value_at_slice(outputs[1].values.as_slice(), index),
        value_at_slice(outputs[2].values.as_slice(), index),
        value_at_slice(outputs[3].values.as_slice(), index),
        value_at_slice(outputs[4].values.as_slice(), index),
    )
}
pub fn midpoint_store(store: &CandleStore, start: usize, end: usize) -> f64 {
    let high = store.high[start..=end]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let low = store.low[start..=end]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    (high + low) / 2.0
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "ICHIMOKU",
        name: "ICHIMOKU",
        category: "Projection",
        pane: "overlay",
        params: vec![
            crate::descriptors::ParamDescriptor {
                name: "tenkan_period",
                label: "Tenkan",
                default: 9.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "kijun_period",
                label: "Kijun",
                default: 26.0,
                min: 1.0,
                step: "1",
            },
            crate::descriptors::ParamDescriptor {
                name: "senkou_b_period",
                label: "Senkou B",
                default: 52.0,
                min: 1.0,
                step: "1",
            },
        ],
        outputs: vec![
            crate::descriptors::output_descriptor("tenkan", "line", "overlay", "#2563eb"),
            crate::descriptors::output_descriptor("kijun", "line", "overlay", "#dc2626"),
            crate::descriptors::output_descriptor("senkou_a", "line", "overlay", "#059669"),
            crate::descriptors::output_descriptor("senkou_b", "line", "overlay", "#ea580c"),
            crate::descriptors::output_descriptor("chikou", "line", "overlay", "#64748b"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlc_store(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            vec![1.0; len],
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
    fn ichimoku_is_flat_for_constant_prices() {
        let store = ohlc_store(&[10.0, 10.0, 10.0, 10.0]);
        let values = ichimoku_store(&store, 2, 2, 2, &mut HashMap::new());

        assert_series_close(&values[0].values, &[f64::NAN, 10.0, 10.0, 10.0]);
        assert_series_close(&values[1].values, &[f64::NAN, 10.0, 10.0, 10.0]);
        assert_series_close(&values[2].values, &[f64::NAN, 10.0, 10.0, 10.0]);
        assert_series_close(&values[3].values, &[f64::NAN, 10.0, 10.0, 10.0]);
        assert_series_close(&values[4].values, &[10.0, 10.0, 10.0, 10.0]);
        assert_eq!(
            latest_ichimoku_store(&store, 2, 2, 2),
            (Some(10.0), Some(10.0), Some(10.0), Some(10.0), Some(10.0))
        );
    }

    #[test]
    fn ichimoku_uses_window_midpoints_on_rising_prices() {
        let store = ohlc_store(&[10.0, 12.0, 14.0, 16.0]);
        let values = ichimoku_store(&store, 2, 2, 2, &mut HashMap::new());

        assert_series_close(&values[0].values, &[f64::NAN, 11.0, 13.0, 15.0]);
        assert_series_close(&values[1].values, &[f64::NAN, 11.0, 13.0, 15.0]);
        assert_series_close(&values[2].values, &[f64::NAN, 11.0, 13.0, 15.0]);
        assert_series_close(&values[3].values, &[f64::NAN, 11.0, 13.0, 15.0]);
        assert_series_close(&values[4].values, &[10.0, 12.0, 14.0, 16.0]);
        assert_eq!(
            latest_ichimoku_store(&store, 2, 2, 2),
            (Some(15.0), Some(15.0), Some(15.0), Some(15.0), Some(16.0))
        );
    }
}

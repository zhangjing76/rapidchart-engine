use crate::output_at_vec;
use crate::NodeCache;
use crate::CandleStore;
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
            crate::named_series("kijun", nodes.get(&kijun_key).map(Rc::clone).unwrap_or_else(|| Rc::new(Vec::new()))),
            crate::named_series("senkou_a", nodes.get(&senkou_a_key).map(Rc::clone).unwrap_or_else(|| Rc::new(Vec::new()))),
            crate::named_series("senkou_b", nodes.get(&senkou_b_key).map(Rc::clone).unwrap_or_else(|| Rc::new(Vec::new()))),
            crate::named_series("chikou", nodes.get("ichimoku:chikou").map(Rc::clone).unwrap_or_else(|| Rc::new(Vec::new()))),
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
        output_at_vec(&outputs, "tenkan", index),
        output_at_vec(&outputs, "kijun", index),
        output_at_vec(&outputs, "senkou_a", index),
        output_at_vec(&outputs, "senkou_b", index),
        output_at_vec(&outputs, "chikou", index),
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

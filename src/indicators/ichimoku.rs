use crate::IndicatorOutput;
use crate::NodeCache;
use crate::{nan_to_none, rc_into_owned};
use crate::{output_at, output_at_vec};
use crate::{Bar, CandleStore, RcSeries, Series};
use std::collections::HashMap;
use std::rc::Rc;

pub fn ichimoku(
    bars: &[Bar],
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let tenkan_key = format!("ichimoku:tenkan:{tenkan_period}");
    let kijun_key = format!("ichimoku:kijun:{kijun_period}");
    let senkou_a_key = format!("ichimoku:senkou_a:{tenkan_period}:{kijun_period}");
    let senkou_b_key = format!("ichimoku:senkou_b:{senkou_b_period}");
    if let Some(values) = nodes.get(&tenkan_key) {
        return vec![
            IndicatorOutput {
                name: "tenkan".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "kijun".to_string(),
                values: nodes
                    .get(&kijun_key)
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "senkou_a".to_string(),
                values: nodes
                    .get(&senkou_a_key)
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "senkou_b".to_string(),
                values: nodes
                    .get(&senkou_b_key)
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "chikou".to_string(),
                values: nodes
                    .get("ichimoku:chikou")
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
        ];
    }
    let mut tenkan = vec![f64::NAN; bars.len()];
    let mut kijun = vec![f64::NAN; bars.len()];
    let mut senkou_a = vec![f64::NAN; bars.len()];
    let mut senkou_b = vec![f64::NAN; bars.len()];
    let chikou: Vec<_> = bars.iter().map(|bar| bar.close).collect();
    for index in 0..bars.len() {
        if index + 1 >= tenkan_period {
            tenkan[index] = midpoint(&bars[index + 1 - tenkan_period..=index]);
        }
        if index + 1 >= kijun_period {
            kijun[index] = midpoint(&bars[index + 1 - kijun_period..=index]);
        }
        let tenkan_value = tenkan[index];
        let kijun_value = kijun[index];
        if !tenkan_value.is_nan() && !kijun_value.is_nan() {
            senkou_a[index] = (tenkan_value + kijun_value) / 2.0;
        }
        if index + 1 >= senkou_b_period {
            senkou_b[index] = midpoint(&bars[index + 1 - senkou_b_period..=index]);
        }
    }
    nodes.insert(tenkan_key, Rc::new(tenkan.clone()));
    nodes.insert(kijun_key, Rc::new(kijun.clone()));
    nodes.insert(senkou_a_key, Rc::new(senkou_a.clone()));
    nodes.insert(senkou_b_key, Rc::new(senkou_b.clone()));
    nodes.insert("ichimoku:chikou".to_string(), Rc::new(chikou.clone()));
    vec![
        IndicatorOutput {
            name: "tenkan".to_string(),
            values: tenkan,
        },
        IndicatorOutput {
            name: "kijun".to_string(),
            values: kijun,
        },
        IndicatorOutput {
            name: "senkou_a".to_string(),
            values: senkou_a,
        },
        IndicatorOutput {
            name: "senkou_b".to_string(),
            values: senkou_b,
        },
        IndicatorOutput {
            name: "chikou".to_string(),
            values: chikou,
        },
    ]
}
pub fn ichimoku_store(
    store: &CandleStore,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let tenkan_key = format!("ichimoku:tenkan:{tenkan_period}");
    let kijun_key = format!("ichimoku:kijun:{kijun_period}");
    let senkou_a_key = format!("ichimoku:senkou_a:{tenkan_period}:{kijun_period}");
    let senkou_b_key = format!("ichimoku:senkou_b:{senkou_b_period}");
    if let Some(values) = nodes.get(&tenkan_key) {
        return vec![
            IndicatorOutput {
                name: "tenkan".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "kijun".to_string(),
                values: nodes
                    .get(&kijun_key)
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "senkou_a".to_string(),
                values: nodes
                    .get(&senkou_a_key)
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "senkou_b".to_string(),
                values: nodes
                    .get(&senkou_b_key)
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "chikou".to_string(),
                values: nodes
                    .get("ichimoku:chikou")
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
        ];
    }
    let mut tenkan = vec![f64::NAN; store.len()];
    let mut kijun = vec![f64::NAN; store.len()];
    let mut senkou_a = vec![f64::NAN; store.len()];
    let mut senkou_b = vec![f64::NAN; store.len()];
    let chikou: Vec<_> = store.close.iter().copied().collect();
    for index in 0..store.len() {
        if index + 1 >= tenkan_period {
            tenkan[index] = midpoint_store(store, index + 1 - tenkan_period, index);
        }
        if index + 1 >= kijun_period {
            kijun[index] = midpoint_store(store, index + 1 - kijun_period, index);
        }
        let tenkan_value = tenkan[index];
        let kijun_value = kijun[index];
        if !tenkan_value.is_nan() && !kijun_value.is_nan() {
            senkou_a[index] = (tenkan_value + kijun_value) / 2.0;
        }
        if index + 1 >= senkou_b_period {
            senkou_b[index] = midpoint_store(store, index + 1 - senkou_b_period, index);
        }
    }
    nodes.insert(tenkan_key, Rc::new(tenkan.clone()));
    nodes.insert(kijun_key, Rc::new(kijun.clone()));
    nodes.insert(senkou_a_key, Rc::new(senkou_a.clone()));
    nodes.insert(senkou_b_key, Rc::new(senkou_b.clone()));
    nodes.insert("ichimoku:chikou".to_string(), Rc::new(chikou.clone()));
    vec![
        IndicatorOutput {
            name: "tenkan".to_string(),
            values: tenkan,
        },
        IndicatorOutput {
            name: "kijun".to_string(),
            values: kijun,
        },
        IndicatorOutput {
            name: "senkou_a".to_string(),
            values: senkou_a,
        },
        IndicatorOutput {
            name: "senkou_b".to_string(),
            values: senkou_b,
        },
        IndicatorOutput {
            name: "chikou".to_string(),
            values: chikou,
        },
    ]
}
pub fn latest_ichimoku(
    bars: &[Bar],
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    let outputs = ichimoku(
        bars,
        tenkan_period,
        kijun_period,
        senkou_b_period,
        &mut HashMap::new(),
    );
    let index = bars.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "tenkan", index),
        output_at_vec(&outputs, "kijun", index),
        output_at_vec(&outputs, "senkou_a", index),
        output_at_vec(&outputs, "senkou_b", index),
        output_at_vec(&outputs, "chikou", index),
    )
}
pub fn latest_ichimoku_store(
    store: &CandleStore,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
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
pub fn midpoint(window: &[Bar]) -> f64 {
    let high = window
        .iter()
        .map(|bar| bar.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = window
        .iter()
        .map(|bar| bar.low)
        .fold(f64::INFINITY, f64::min);
    (high + low) / 2.0
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

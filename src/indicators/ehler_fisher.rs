use crate::NodeCache;
use crate::{Bar, CandleStore, IndicatorOutput};
use std::collections::HashMap;

/// Ehlers Fisher Transform:
/// 1. Normalize price to -1..+1 range over period using (H+L)/2
/// 2. Apply Fisher Transform: fisher = 0.5 * ln((1+x)/(1-x))
/// 3. Outputs: fisher line and trigger (previous fisher value)
pub fn ehler_fisher(bars: &[Bar], period: usize, _nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let len = bars.len();
    let mut fisher_out = vec![f64::NAN; len];
    let mut trigger_out = vec![f64::NAN; len];
    if period < 2 || len < period {
        return vec![
            IndicatorOutput { name: "fisher".to_string(), values: fisher_out },
            IndicatorOutput { name: "trigger".to_string(), values: trigger_out },
        ];
    }
    let mut prev_value = 0.0f64;
    let mut prev_fisher = 0.0f64;
    for i in period - 1..len {
        let window = &bars[i + 1 - period..=i];
        let max_high = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let min_low = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let mid = (bars[i].high + bars[i].low) / 2.0;
        let range = max_high - min_low;
        let normalized = if range > 1e-10 {
            0.33 * 2.0 * ((mid - min_low) / range - 0.5) + 0.67 * prev_value
        } else {
            0.67 * prev_value
        };
        let clamped = normalized.max(-0.999).min(0.999);
        let fisher = 0.5 * ((1.0 + clamped) / (1.0 - clamped)).ln() + 0.5 * prev_fisher;
        fisher_out[i] = fisher;
        trigger_out[i] = prev_fisher;
        prev_value = clamped;
        prev_fisher = fisher;
    }
    vec![
        IndicatorOutput { name: "fisher".to_string(), values: fisher_out },
        IndicatorOutput { name: "trigger".to_string(), values: trigger_out },
    ]
}

pub fn ehler_fisher_store(store: &CandleStore, period: usize, _nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let len = store.len();
    let mut fisher_out = vec![f64::NAN; len];
    let mut trigger_out = vec![f64::NAN; len];
    if period < 2 || len < period {
        return vec![
            IndicatorOutput { name: "fisher".to_string(), values: fisher_out },
            IndicatorOutput { name: "trigger".to_string(), values: trigger_out },
        ];
    }
    let mut prev_value = 0.0f64;
    let mut prev_fisher = 0.0f64;
    for i in period - 1..len {
        let max_high = store.high[i + 1 - period..=i].iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_low = store.low[i + 1 - period..=i].iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let mid = (store.high[i] + store.low[i]) / 2.0;
        let range = max_high - min_low;
        let normalized = if range > 1e-10 {
            0.33 * 2.0 * ((mid - min_low) / range - 0.5) + 0.67 * prev_value
        } else {
            0.67 * prev_value
        };
        let clamped = normalized.max(-0.999).min(0.999);
        let fisher = 0.5 * ((1.0 + clamped) / (1.0 - clamped)).ln() + 0.5 * prev_fisher;
        fisher_out[i] = fisher;
        trigger_out[i] = prev_fisher;
        prev_value = clamped;
        prev_fisher = fisher;
    }
    vec![
        IndicatorOutput { name: "fisher".to_string(), values: fisher_out },
        IndicatorOutput { name: "trigger".to_string(), values: trigger_out },
    ]
}

pub fn latest_ehler_fisher_store(store: &CandleStore, period: usize) -> (Option<f64>, Option<f64>) {
    let outputs = ehler_fisher_store(store, period, &mut HashMap::new());
    let fisher = outputs[0].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    let trigger = outputs[1].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    (fisher, trigger)
}

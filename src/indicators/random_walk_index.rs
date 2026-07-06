use crate::NodeCache;
use crate::{CandleStore, IndicatorOutput};
use std::collections::HashMap;

/// Random Walk Index:
/// Measures whether price movement is random or trending.
/// RWI_High = (High - Low[n]) / (ATR * sqrt(n))
/// RWI_Low = (High[n] - Low) / (ATR * sqrt(n))
/// where n ranges from 2 to period, and we take the maximum.
/// High RWI (>1) suggests trending; low (<1) suggests random.

pub fn random_walk_index_store(store: &CandleStore, period: usize, _nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let len = store.len();
    let mut rw_high = vec![f64::NAN; len];
    let mut rw_low = vec![f64::NAN; len];
    if period < 2 || len < period {
        return vec![
            IndicatorOutput { name: "high".to_string(), values: rw_high },
            IndicatorOutput { name: "low".to_string(), values: rw_low },
        ];
    }
    let mut tr = vec![0.0f64; len];
    for i in 1..len {
        tr[i] = (store.high[i] - store.low[i])
            .max((store.high[i] - store.close[i - 1]).abs())
            .max((store.low[i] - store.close[i - 1]).abs());
    }
    for i in period..len {
        let mut max_rwi_high = 0.0f64;
        let mut max_rwi_low = 0.0f64;
        for n in 2..=period {
            if i < n { break; }
            let atr_n: f64 = tr[i + 1 - n..=i].iter().sum::<f64>() / n as f64;
            let denom = atr_n * (n as f64).sqrt();
            if denom > 1e-10 {
                let rwi_h = (store.high[i] - store.low[i - n]) / denom;
                let rwi_l = (store.high[i - n] - store.low[i]) / denom;
                max_rwi_high = max_rwi_high.max(rwi_h);
                max_rwi_low = max_rwi_low.max(rwi_l);
            }
        }
        rw_high[i] = max_rwi_high;
        rw_low[i] = max_rwi_low;
    }
    vec![
        IndicatorOutput { name: "high".to_string(), values: rw_high },
        IndicatorOutput { name: "low".to_string(), values: rw_low },
    ]
}

pub fn latest_random_walk_index_store(store: &CandleStore, period: usize) -> (Option<f64>, Option<f64>) {
    let outputs = random_walk_index_store(store, period, &mut HashMap::new());
    let h = outputs[0].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    let l = outputs[1].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    (h, l)
}
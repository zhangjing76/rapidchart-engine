use crate::indicators::true_range::true_range_series_store;
use crate::CandleStore;
use crate::NodeCache;
use std::collections::HashMap;

/// Random Walk Index (Poulos), the largest value over lookbacks n = 1..=period of
/// RWI_High = (High - Low[n]) / (ATR_n * sqrt(n))
/// RWI_Low = (High[n] - Low) / (ATR_n * sqrt(n))
/// where ATR_n is the average true range of the n bars before the current one. Above 1 suggests a
/// trend; below 1 a random walk. Bar 0 has no true range, so the first value is at bar
/// `period + 1`.
pub fn random_walk_index_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let len = store.len();
    let mut rw_high = vec![f64::NAN; len];
    let mut rw_low = vec![f64::NAN; len];
    if period == 0 || len <= period + 1 {
        return vec![
            crate::named_series("high", rw_high),
            crate::named_series("low", rw_low),
        ];
    }
    let tr = true_range_series_store(store, nodes);
    for i in period + 1..len {
        let mut max_rwi_high = 0.0f64;
        let mut max_rwi_low = 0.0f64;
        let mut tr_sum = 0.0;
        for n in 1..=period {
            tr_sum += tr[i - n];
            let denom = tr_sum / n as f64 * (n as f64).sqrt();
            if denom > 1e-10 {
                max_rwi_high = max_rwi_high.max((store.high[i] - store.low[i - n]) / denom);
                max_rwi_low = max_rwi_low.max((store.high[i - n] - store.low[i]) / denom);
            }
        }
        rw_high[i] = max_rwi_high;
        rw_low[i] = max_rwi_low;
    }
    vec![
        crate::named_series("high", rw_high),
        crate::named_series("low", rw_low),
    ]
}
pub fn latest_random_walk_index_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>) {
    let outputs = random_walk_index_store(store, period, &mut HashMap::new());
    let h = outputs[0]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    let l = outputs[1]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    (h, l)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "RANDOM_WALK_INDEX",
        name: "RANDOM WALK INDEX",
        category: "Statistical",
        pane: "separate",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "period",
            label: "Period",
            default: 14.0,
            min: 2.0,
            step: "1",
        }],
        outputs: vec![
            crate::descriptors::output_descriptor("high", "line", "separate", "#059669"),
            crate::descriptors::output_descriptor("low", "line", "separate", "#dc2626"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn random_walk_index_is_one_for_identical_bars() {
        let store = ohlc_store(&[
            (3.0, 0.0, 1.0),
            (3.0, 0.0, 1.0),
            (3.0, 0.0, 1.0),
            (3.0, 0.0, 1.0),
        ]);
        let values = random_walk_index_store(&store, 2, &mut HashMap::new());

        // Every true range is 3, so the one-bar lookback gives 3 / 3 = 1 and the two-bar one
        // 3 / (3 * sqrt(2)).
        assert_series_close(&values[0].values, &[f64::NAN, f64::NAN, f64::NAN, 1.0]);
        assert_series_close(&values[1].values, &[f64::NAN, f64::NAN, f64::NAN, 1.0]);
        assert_eq!(
            latest_random_walk_index_store(&store, 2),
            (Some(1.0), Some(1.0))
        );
    }

    #[test]
    fn random_walk_index_detects_an_uptrend_more_than_a_downtrend() {
        let store = ohlc_store(&[
            (2.0, 1.0, 1.5),
            (3.0, 2.0, 2.5),
            (4.0, 3.0, 3.5),
            (5.0, 4.0, 4.5),
        ]);
        let values = random_walk_index_store(&store, 2, &mut HashMap::new());

        // The true ranges of bars 1 and 2 are 1.5. Over two bars the high of 5 is 3 above the
        // low of bar 1: 3 / (1.5 * sqrt(2)) = sqrt(2), more than the one-bar 2 / 1.5.
        assert_series_close(
            &values[0].values,
            &[f64::NAN, f64::NAN, f64::NAN, std::f64::consts::SQRT_2],
        );
        assert_series_close(&values[1].values, &[f64::NAN, f64::NAN, f64::NAN, 0.0]);
    }
}

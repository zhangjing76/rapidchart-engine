use crate::CandleStore;
use crate::NodeCache;
use std::collections::HashMap;

/// Shinohara Intensity Ratio:
/// Strong Ratio = SUM(H - C) / SUM(C - L) * 100 over period
/// Weak Ratio = SUM(H - PC) / SUM(PC - L) * 100 over period
/// where PC = previous close
pub fn shinohara_intensity_store(
    store: &CandleStore,
    period: usize,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let len = store.len();
    let mut strong = vec![f64::NAN; len];
    let mut weak = vec![f64::NAN; len];
    if period == 0 || len < period + 1 {
        return vec![
            crate::named_series("strong", strong),
            crate::named_series("weak", weak),
        ];
    }
    for i in period..len {
        let mut sum_hc = 0.0;
        let mut sum_cl = 0.0;
        let mut sum_hpc = 0.0;
        let mut sum_pcl = 0.0;
        for j in i + 1 - period..=i {
            sum_hc += store.high[j] - store.close[j];
            sum_cl += store.close[j] - store.low[j];
            let pc = store.close[j - 1];
            sum_hpc += store.high[j] - pc;
            sum_pcl += pc - store.low[j];
        }
        if sum_cl.abs() > 1e-10 {
            strong[i] = (sum_hc / sum_cl) * 100.0;
        }
        if sum_pcl.abs() > 1e-10 {
            weak[i] = (sum_hpc / sum_pcl) * 100.0;
        }
    }
    vec![
        crate::named_series("strong", strong),
        crate::named_series("weak", weak),
    ]
}
pub fn latest_shinohara_intensity_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>) {
    let o = shinohara_intensity_store(store, period, &mut HashMap::new());
    let s = o[0]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    let w = o[1]
        .values
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) });
    (s, w)
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "SHINOHARA_INTENSITY",
                name: "SHINOHARA INTENSITY RATIO",
                category: "Trend Analysis",
                pane: "separate",
                params: vec![crate::descriptors::ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 26.0,
                    min: 1.0,
                    step: "1",
                }],
                outputs: vec![
                    crate::descriptors::output_descriptor("strong", "line", "separate", "#2563eb"),
                    crate::descriptors::output_descriptor("weak", "line", "separate", "#dc2626"),
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

    #[test]
    fn shinohara_intensity_is_hundred_for_midpoint_closes() {
        let store = ohlc_store(&[(2.0, 0.0, 1.0), (2.0, 0.0, 1.0), (2.0, 0.0, 1.0)]);
        let values = shinohara_intensity_store(&store, 2, &mut std::collections::HashMap::new());

        assert!(values[0].values[0].is_nan());
        assert!((values[0].values[2] - 100.0).abs() < 1e-12);
        assert!((values[1].values[2] - 100.0).abs() < 1e-12);
        assert_eq!(
            latest_shinohara_intensity_store(&store, 2),
            (Some(100.0), Some(100.0))
        );
    }
}

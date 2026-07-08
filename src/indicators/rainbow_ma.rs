use crate::NodeCache;
use crate::{CandleStore, Series};

/// Rainbow Moving Average: 10 recursive SMAs.
/// Layer 1 = SMA(close, period)
/// Layer 2 = SMA(Layer 1, period)
/// ... Layer 10 = SMA(Layer 9, period)
///
/// Each layer is an SMA of the previous layer's output.
fn sma_of_series(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    // Build running sum skipping NaNs properly
    let mut valid_start = 0;
    // Find first index where we have `period` consecutive non-NaN values
    let mut count = 0;
    for (i, &v) in values.iter().enumerate() {
        if v.is_nan() {
            count = 0;
        } else {
            count += 1;
            if count == period {
                valid_start = i + 1 - period;
                break;
            }
        }
    }
    if count < period {
        return out;
    }
    let start_idx = valid_start + period - 1;
    let mut sum: f64 = values[valid_start..=start_idx].iter().sum();
    out[start_idx] = sum / period as f64;
    for i in start_idx + 1..values.len() {
        if values[i].is_nan() {
            out[i] = f64::NAN;
        } else {
            sum += values[i] - values[i - period];
            out[i] = sum / period as f64;
        }
    }
    out
}

pub fn rainbow_ma_store(
    store: &CandleStore,
    period: usize,
    _nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let mut layers: Vec<Series> = Vec::with_capacity(10);
    let first = sma_of_series(&store.close, period);
    layers.push(first);
    for i in 1..10 {
        let prev = &layers[i - 1];
        layers.push(sma_of_series(prev, period));
    }
    layers
        .into_iter()
        .enumerate()
        .map(|(i, values)| crate::named_series(format!("r{}", i + 1), values))
        .collect()
}

pub fn latest_rainbow_ma_store(store: &CandleStore, period: usize) -> Vec<(String, Option<f64>)> {
    let mut layers: Vec<Series> = Vec::with_capacity(10);
    let first = sma_of_series(&store.close, period);
    layers.push(first);
    for i in 1..10 {
        let prev = &layers[i - 1];
        layers.push(sma_of_series(prev, period));
    }
    layers
        .iter()
        .enumerate()
        .map(|(i, values)| {
            let last = values
                .last()
                .copied()
                .and_then(|v| if v.is_nan() { None } else { Some(v) });
            (format!("r{}", i + 1), last)
        })
        .collect()
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "RAINBOW_MA",
        name: "RAINBOW MOVING AVERAGE",
        category: "Averages/Bands",
        pane: "overlay",
        params: vec![crate::descriptors::ParamDescriptor {
            name: "period",
            label: "Period",
            default: 2.0,
            min: 1.0,
            step: "1",
        }],
        outputs: vec![
            crate::descriptors::output_descriptor("r1", "line", "overlay", "#dc2626"),
            crate::descriptors::output_descriptor("r2", "line", "overlay", "#ea580c"),
            crate::descriptors::output_descriptor("r3", "line", "overlay", "#f59e0b"),
            crate::descriptors::output_descriptor("r4", "line", "overlay", "#84cc16"),
            crate::descriptors::output_descriptor("r5", "line", "overlay", "#059669"),
            crate::descriptors::output_descriptor("r6", "line", "overlay", "#0891b2"),
            crate::descriptors::output_descriptor("r7", "line", "overlay", "#2563eb"),
            crate::descriptors::output_descriptor("r8", "line", "overlay", "#7c3aed"),
            crate::descriptors::output_descriptor("r9", "line", "overlay", "#c026d3"),
            crate::descriptors::output_descriptor("r10", "line", "overlay", "#9333ea"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close_store(values: &[f64]) -> CandleStore {
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
    fn rainbow_ma_stays_flat_for_constant_prices() {
        let store = close_store(&[10.0; 6]);
        let values = rainbow_ma_store(&store, 1, &mut std::collections::HashMap::new());
        for series in &values {
            assert_series_close(&series.values, &[10.0, 10.0, 10.0, 10.0, 10.0, 10.0]);
        }
        let latest = latest_rainbow_ma_store(&store, 1);
        for (_, value) in latest {
            assert_eq!(value, Some(10.0));
        }
    }

    #[test]
    fn rainbow_ma_builds_recursive_sma_layers() {
        let store = close_store(&[10.0, 12.0, 14.0, 16.0, 18.0, 20.0]);
        let values = rainbow_ma_store(&store, 2, &mut std::collections::HashMap::new());

        assert_series_close(&values[0].values, &[f64::NAN, 11.0, 13.0, 15.0, 17.0, 19.0]);
        assert_series_close(
            &values[1].values,
            &[f64::NAN, f64::NAN, 12.0, 14.0, 16.0, 18.0],
        );
        assert_series_close(
            &values[2].values,
            &[f64::NAN, f64::NAN, f64::NAN, 13.0, 15.0, 17.0],
        );
        assert_series_close(
            &values[3].values,
            &[f64::NAN, f64::NAN, f64::NAN, f64::NAN, 14.0, 16.0],
        );

        let latest = latest_rainbow_ma_store(&store, 2);
        assert_eq!(latest[0], ("r1".to_string(), Some(19.0)));
        assert_eq!(latest[1], ("r2".to_string(), Some(18.0)));
        assert_eq!(latest[2], ("r3".to_string(), Some(17.0)));
        assert_eq!(latest[3], ("r4".to_string(), Some(16.0)));
    }
}

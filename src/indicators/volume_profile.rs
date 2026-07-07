use crate::indicators::derived::hl2_store;
use crate::NodeCache;
use crate::{CandleStore, IndicatorOutput};
use std::collections::HashMap;

/// Volume Profile (simplified for line chart output):
/// Identifies the Point of Control (POC) — the price level with the highest
/// volume over the lookback period — and Value Area High/Low (70% of volume).
///
/// Outputs: poc (price with most volume), vah (value area high), val (value area low)

pub fn volume_profile_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let len = store.len();
    let hl2 = hl2_store(store, nodes);
    let mut poc_out = vec![f64::NAN; len];
    let mut vah_out = vec![f64::NAN; len];
    let mut val_out = vec![f64::NAN; len];
    if period == 0 || len < period {
        return vec![
            IndicatorOutput { name: "poc".to_string(), values: poc_out },
            IndicatorOutput { name: "vah".to_string(), values: vah_out },
            IndicatorOutput { name: "val".to_string(), values: val_out },
        ];
    }
    let num_bins = 20usize;
    for i in period - 1..len {
        let window_start = i + 1 - period;
        let mut high = f64::NEG_INFINITY;
        let mut low = f64::INFINITY;
        for j in window_start..=i {
            high = high.max(store.high[j]);
            low = low.min(store.low[j]);
        }
        let range = high - low;
        if range < 1e-10 {
            poc_out[i] = (high + low) / 2.0;
            vah_out[i] = high;
            val_out[i] = low;
            continue;
        }
        let bin_size = range / num_bins as f64;
        let mut bins = vec![0.0f64; num_bins];
        let mut total_vol = 0.0f64;
        for j in window_start..=i {
            let mid = hl2[j];
            let bin = ((mid - low) / bin_size).floor() as usize;
            let bin = bin.min(num_bins - 1);
            bins[bin] += store.volume[j];
            total_vol += store.volume[j];
        }
        let poc_bin = bins.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        poc_out[i] = low + (poc_bin as f64 + 0.5) * bin_size;
        let target_vol = total_vol * 0.7;
        let mut va_vol = bins[poc_bin];
        let mut va_low_bin = poc_bin;
        let mut va_high_bin = poc_bin;
        while va_vol < target_vol {
            let expand_up = if va_high_bin + 1 < num_bins { bins[va_high_bin + 1] } else { 0.0 };
            let expand_down = if va_low_bin > 0 { bins[va_low_bin - 1] } else { 0.0 };
            if expand_up >= expand_down && va_high_bin + 1 < num_bins {
                va_high_bin += 1;
                va_vol += bins[va_high_bin];
            } else if va_low_bin > 0 {
                va_low_bin -= 1;
                va_vol += bins[va_low_bin];
            } else if va_high_bin + 1 < num_bins {
                va_high_bin += 1;
                va_vol += bins[va_high_bin];
            } else { break; }
        }
        vah_out[i] = low + (va_high_bin as f64 + 1.0) * bin_size;
        val_out[i] = low + va_low_bin as f64 * bin_size;
    }
    vec![
        IndicatorOutput { name: "poc".to_string(), values: poc_out },
        IndicatorOutput { name: "vah".to_string(), values: vah_out },
        IndicatorOutput { name: "val".to_string(), values: val_out },
    ]
}

pub fn latest_volume_profile_store(store: &CandleStore, period: usize) -> (Option<f64>, Option<f64>, Option<f64>) {
    let outputs = volume_profile_store(store, period, &mut HashMap::new());
    let poc = outputs[0].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    let vah = outputs[1].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    let val = outputs[2].values.last().copied().and_then(|v| if v.is_nan() { None } else { Some(v) });
    (poc, vah, val)
}
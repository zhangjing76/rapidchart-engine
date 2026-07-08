use crate::indicators::ema::ema_series;
use crate::NodeCache;
use crate::{CandleStore, RcSeries};
use std::collections::HashMap;
use std::rc::Rc;

/// Klinger Volume Oscillator:
/// Volume Force (VF) = volume * |2*(dm/cm) - 1| * trend * 100
/// where trend = +1 if (H+L+C) > prev(H+L+C), else -1
/// dm = high - low, cm = cumulative dm in same trend direction
/// KVO = EMA(34, VF) - EMA(55, VF)
pub fn klinger_volume_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "klinger:hlcv".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if len < 2 {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let mut vf = vec![0.0f64; len];
    let mut cm = 0.0f64;
    let mut prev_trend = 1i8;
    for i in 1..len {
        let hlc = store.high[i] + store.low[i] + store.close[i];
        let prev_hlc = store.high[i - 1] + store.low[i - 1] + store.close[i - 1];
        let trend: i8 = if hlc > prev_hlc { 1 } else { -1 };
        let dm = store.high[i] - store.low[i];
        if trend == prev_trend {
            cm += dm;
        } else {
            cm = dm;
        }
        let ratio = if cm.abs() > 1e-10 {
            (2.0 * dm / cm) - 1.0
        } else {
            0.0
        };
        vf[i] = store.volume[i] * ratio.abs() * trend as f64 * 100.0;
        prev_trend = trend;
    }
    let ema34 = ema_series(&vf, 34);
    let ema55 = ema_series(&vf, 55);
    for i in 0..len {
        if !ema34[i].is_nan() && !ema55[i].is_nan() {
            out[i] = ema34[i] - ema55[i];
        }
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

pub fn latest_klinger_volume_store(store: &CandleStore) -> Option<f64> {
    klinger_volume_store(store, &mut HashMap::new())
        .last()
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
                kind: "KLINGER_VOLUME",
                name: "KLINGER VOLUME OSCILLATOR",
                category: "Money Flow",
                pane: "separate",
                params: Vec::new(),
                outputs: vec![crate::descriptors::output_descriptor("value", "line", "separate", "#2563eb")],
            }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ohlcv_store(values: &[(f64, f64, f64, f64)]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(high, _, _, _)| *high).collect(),
            values.iter().map(|(_, low, _, _)| *low).collect(),
            values.iter().map(|(_, _, close, _)| *close).collect(),
            values.iter().map(|(_, _, _, volume)| *volume).collect(),
        )
    }

    #[test]
    fn klinger_volume_is_zero_for_constant_prices_and_volume() {
        let store = ohlcv_store(&[
            (10.0, 10.0, 10.0, 5.0),
            (10.0, 10.0, 10.0, 5.0),
            (10.0, 10.0, 10.0, 5.0),
        ]);
        let values = klinger_volume_store(&store, &mut HashMap::new());

        assert_eq!(&*values, &[0.0, 0.0, 0.0]);
        assert_eq!(latest_klinger_volume_store(&store), Some(0.0));
    }

    #[test]
    fn klinger_volume_tracks_the_gap_between_fast_and_slow_vf_emas() {
        let input: Vec<_> = (0..60)
            .map(|i| {
                let base = 10.0 + i as f64;
                (base + 1.0, base, base + 0.5, 1.0)
            })
            .collect();
        let store = ohlcv_store(&input);
        let values = klinger_volume_store(&store, &mut HashMap::new());

        assert!((values[59] - 9.580894095402968).abs() < 1e-12);
        assert_eq!(latest_klinger_volume_store(&store), Some(9.580894095402968));
    }
}

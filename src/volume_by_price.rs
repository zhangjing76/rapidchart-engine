use crate::CandleStore;

/// Volume traded at each price level over a range of bars, split into buckets of equal price
/// size between the range's lowest low and highest high. Bucket `i` covers
/// `low + i * size .. low + (i + 1) * size`, where `size = (high - low) / buckets`.
#[derive(Debug, PartialEq)]
pub(crate) struct VolumeByPrice {
    pub low: f64,
    pub high: f64,
    /// Volume from bars that closed at or above their open.
    pub up: Vec<f64>,
    /// Volume from bars that closed below their open.
    pub down: Vec<f64>,
}

/// Distribute the volume of bars `from..=to` across `buckets` price buckets.
///
/// Each bar's volume is spread evenly over its high-low range, so a bucket gets the share of the
/// bar's range it overlaps. A bar with no range puts all its volume in the bucket of its price.
/// `to` is clamped to the last bar. Returns None for an empty range or zero buckets.
pub(crate) fn volume_by_price(
    store: &CandleStore,
    from: usize,
    to: usize,
    buckets: usize,
) -> Option<VolumeByPrice> {
    let len = store.len();
    if buckets == 0 || len == 0 || from >= len || from > to {
        return None;
    }
    let to = to.min(len - 1);
    let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
    for i in from..=to {
        low = low.min(store.low[i]);
        high = high.max(store.high[i]);
    }
    if !low.is_finite() || !high.is_finite() {
        return None;
    }

    let mut up = vec![0.0; buckets];
    let mut down = vec![0.0; buckets];
    let span = high - low;
    let size = span / buckets as f64;
    let bucket_of = |price: f64| -> usize {
        if span <= 0.0 {
            0
        } else {
            (((price - low) / size).floor().max(0.0) as usize).min(buckets - 1)
        }
    };

    for i in from..=to {
        let volume = store.volume[i];
        // Skips zero, negative and NaN volume.
        if volume.is_nan() || volume <= 0.0 {
            continue;
        }
        let target = if store.close[i] >= store.open[i] {
            &mut up
        } else {
            &mut down
        };
        let (bar_low, bar_high) = (store.low[i], store.high[i]);
        let first = bucket_of(bar_low);
        let last = bucket_of(bar_high);
        if bar_high <= bar_low || first == last {
            target[bucket_of((bar_low + bar_high) / 2.0)] += volume;
            continue;
        }
        let range = bar_high - bar_low;
        for (bucket, total) in target.iter_mut().enumerate().take(last + 1).skip(first) {
            let bucket_low = low + bucket as f64 * size;
            let overlap = bar_high.min(bucket_low + size) - bar_low.max(bucket_low);
            if overlap > 0.0 {
                *total += volume * overlap / range;
            }
        }
    }

    Some(VolumeByPrice {
        low,
        high,
        up,
        down,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bar;

    fn store(bars: &[(f64, f64, f64, f64, f64)]) -> CandleStore {
        let mut store = CandleStore::default();
        for (i, &(open, high, low, close, volume)) in bars.iter().enumerate() {
            store.push(Bar {
                time: i as u32,
                open,
                high,
                low,
                close,
                volume,
            });
        }
        store
    }

    fn close_to(a: &[f64], b: &[f64]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9)
    }

    #[test]
    fn spreads_each_bar_over_its_range_and_splits_up_and_down() {
        // Range 10..14 in 4 buckets of 1.
        let s = store(&[
            (10.0, 14.0, 10.0, 13.0, 400.0), // up, covers all four buckets
            (12.0, 12.0, 11.0, 11.0, 100.0), // down, covers bucket 1 only
        ]);
        let profile = volume_by_price(&s, 0, 1, 4).unwrap();
        assert_eq!((profile.low, profile.high), (10.0, 14.0));
        assert!(close_to(&profile.up, &[100.0, 100.0, 100.0, 100.0]));
        assert!(close_to(&profile.down, &[0.0, 100.0, 0.0, 0.0]));
    }

    #[test]
    fn total_volume_is_preserved() {
        let s = store(&[
            (10.0, 10.7, 9.2, 10.4, 250.0),
            (10.4, 11.3, 10.1, 10.2, 175.0),
            (10.2, 12.9, 10.0, 12.5, 90.0),
        ]);
        let profile = volume_by_price(&s, 0, 2, 7).unwrap();
        let total: f64 = profile.up.iter().chain(&profile.down).sum();
        assert!((total - 515.0).abs() < 1e-9);
    }

    #[test]
    fn a_flat_bar_goes_into_the_bucket_of_its_price() {
        let s = store(&[
            (10.0, 14.0, 10.0, 14.0, 1.0),
            (12.5, 12.5, 12.5, 12.5, 50.0),
        ]);
        let profile = volume_by_price(&s, 0, 1, 4).unwrap();
        assert!((profile.up[2] - 50.25).abs() < 1e-9);
    }

    #[test]
    fn all_bars_at_one_price_use_the_first_bucket() {
        let s = store(&[(5.0, 5.0, 5.0, 5.0, 10.0), (5.0, 5.0, 5.0, 5.0, 20.0)]);
        let profile = volume_by_price(&s, 0, 1, 3).unwrap();
        assert!(close_to(&profile.up, &[30.0, 0.0, 0.0]));
    }

    #[test]
    fn uses_only_the_requested_bars_and_clamps_the_end() {
        let s = store(&[
            (1.0, 100.0, 1.0, 50.0, 999.0),
            (10.0, 12.0, 10.0, 11.0, 20.0),
            (11.0, 12.0, 10.0, 10.5, 10.0),
        ]);
        let profile = volume_by_price(&s, 1, 99, 2).unwrap();
        assert_eq!((profile.low, profile.high), (10.0, 12.0));
        assert!(close_to(&profile.up, &[10.0, 10.0]));
        assert!(close_to(&profile.down, &[5.0, 5.0]));
    }

    #[test]
    fn rejects_empty_ranges() {
        let s = store(&[(1.0, 2.0, 1.0, 2.0, 1.0)]);
        assert_eq!(volume_by_price(&s, 0, 0, 0), None);
        assert_eq!(volume_by_price(&s, 1, 2, 5), None);
        assert_eq!(volume_by_price(&s, 0, 0, 5).map(|p| p.up.len()), Some(5));
        assert_eq!(volume_by_price(&CandleStore::default(), 0, 0, 5), None);
    }
}

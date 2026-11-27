use crate::CandleStore;
use crate::NodeCache;
use std::rc::Rc;

/// Pivot, R1, S1, R2, S2, R3, S3 for one bar.
type PivotLevels = [Option<f64>; 7];

const OUTPUT_NAMES: [&str; 7] = ["pp", "r1", "s1", "r2", "s2", "r3", "s3"];
const SECONDS_PER_DAY: u32 = 86_400;

/// How long each pivot period is, chosen from the bar size as ViewPoint (ChartIQ) does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PivotPeriod {
    /// Bars under a minute.
    FifteenMinutes,
    /// Intraday bars under 30 minutes.
    Day,
    /// Intraday bars of 30 minutes or more.
    Week,
    /// Daily bars.
    Month,
    /// Weekly and monthly bars.
    Year,
}

/// The smallest gap between bars, in seconds (a day when there are fewer than two bars).
fn bar_seconds(times: &[u32]) -> u32 {
    times
        .windows(2)
        .filter_map(|w| w[1].checked_sub(w[0]).filter(|gap| *gap > 0))
        .min()
        .unwrap_or(SECONDS_PER_DAY)
}

fn pivot_period(times: &[u32]) -> PivotPeriod {
    match bar_seconds(times) {
        s if s < 60 => PivotPeriod::FifteenMinutes,
        s if s < 30 * 60 => PivotPeriod::Day,
        s if s < SECONDS_PER_DAY => PivotPeriod::Week,
        s if s < 7 * SECONDS_PER_DAY => PivotPeriod::Month,
        _ => PivotPeriod::Year,
    }
}

/// Calendar date of a day count since 1970-01-01 (Howard Hinnant's civil_from_days).
fn civil_date(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Day of the week, with Sunday as 0.
fn weekday(days: i64) -> i64 {
    (days + 4).rem_euclid(7)
}

/// Whether bar `index` starts a new pivot period. Bar times are the exchange's wall-clock time
/// read as UTC, so calendar boundaries are exchange days, weeks, months and years.
fn starts_new_period(period: PivotPeriod, previous: u32, current: u32) -> bool {
    let (pd, cd) = (
        i64::from(previous / SECONDS_PER_DAY),
        i64::from(current / SECONDS_PER_DAY),
    );
    match period {
        PivotPeriod::FifteenMinutes => previous / 900 != current / 900,
        // As in ChartIQ, a Sunday session joins the following day.
        PivotPeriod::Day => pd != cd && weekday(pd) != 0,
        // Weeks start on Sunday.
        PivotPeriod::Week => weekday(cd) < weekday(pd),
        PivotPeriod::Month => {
            let (py, pm, _) = civil_date(pd);
            let (cy, cm, _) = civil_date(cd);
            (py, pm) != (cy, cm)
        }
        PivotPeriod::Year => civil_date(pd).0 != civil_date(cd).0,
    }
}

/// Standard floor pivots from a period's high, low and close.
fn levels(high: f64, low: f64, close: f64) -> PivotLevels {
    let pivot = (high + low + close) / 3.0;
    let range = high - low;
    [
        Some(pivot),
        Some(2.0 * pivot - low),
        Some(2.0 * pivot - high),
        Some(pivot + range),
        Some(pivot - range),
        Some(high + 2.0 * (pivot - low)),
        Some(low - 2.0 * (high - pivot)),
    ]
}

/// Pivot levels for every bar: each bar uses the high, low and close of the previous period.
/// Bars in the first period have no previous period and so no levels.
fn pivot_levels(store: &CandleStore) -> Vec<PivotLevels> {
    let len = store.len();
    let mut out = vec![[None; 7]; len];
    if len < 2 {
        return out;
    }
    let period = pivot_period(&store.time);
    let mut current: PivotLevels = [None; 7];
    let mut high = store.high[0];
    let mut low = store.low[0];
    for index in 1..len {
        if starts_new_period(period, store.time[index - 1], store.time[index]) {
            current = levels(high, low, store.close[index - 1]);
            high = store.high[index];
            low = store.low[index];
        } else {
            high = high.max(store.high[index]);
            low = low.min(store.low[index]);
        }
        out[index] = current;
    }
    out
}

pub fn pivot_points_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
    let rows = pivot_levels(store);
    OUTPUT_NAMES
        .iter()
        .enumerate()
        .map(|(slot, name)| {
            let values: Vec<f64> = rows.iter().map(|r| r[slot].unwrap_or(f64::NAN)).collect();
            nodes.insert(format!("pivot:{name}"), Rc::new(values.clone()));
            crate::named_series(*name, values)
        })
        .collect()
}

/// Levels for the last bar. Recalculated from all bars, since a live bar can start a new period.
pub fn latest_pivot_points_store(store: &CandleStore) -> PivotLevels {
    pivot_levels(store).last().copied().unwrap_or([None; 7])
}

pub(crate) fn descriptor() -> crate::descriptors::IndicatorDescriptor {
    crate::descriptors::IndicatorDescriptor {
        kind: "PIVOT_POINTS",
        name: "PIVOT POINTS",
        category: "Support/Resistance",
        pane: "overlay",
        params: Vec::new(),
        outputs: vec![
            crate::descriptors::output_descriptor("pp", "line", "overlay", "#64748b"),
            crate::descriptors::output_descriptor("r1", "line", "overlay", "#059669"),
            crate::descriptors::output_descriptor("s1", "line", "overlay", "#dc2626"),
            crate::descriptors::output_descriptor("r2", "line", "overlay", "#16a34a"),
            crate::descriptors::output_descriptor("s2", "line", "overlay", "#b91c1c"),
            crate::descriptors::output_descriptor("r3", "line", "overlay", "#15803d"),
            crate::descriptors::output_descriptor("s3", "line", "overlay", "#991b1b"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const DAY: u32 = SECONDS_PER_DAY;

    fn store_at(times: &[u32], values: &[(f64, f64, f64)]) -> CandleStore {
        CandleStore::from_raw_columns(
            times.to_vec(),
            values.iter().map(|(_, _, close)| *close).collect(),
            values.iter().map(|(high, _, _)| *high).collect(),
            values.iter().map(|(_, low, _)| *low).collect(),
            values.iter().map(|(_, _, close)| *close).collect(),
            vec![1.0; values.len()],
        )
    }

    fn assert_series_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan(), "expected NaN, got {actual}");
            } else {
                assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
            }
        }
    }

    #[test]
    fn civil_dates_and_weekdays() {
        assert_eq!(civil_date(0), (1970, 1, 1));
        assert_eq!(civil_date(20_779), (2026, 11, 22));
        assert_eq!(weekday(0), 4); // Thursday
        assert_eq!(weekday(20_779), 0); // Sunday 22 Nov 2026
    }

    #[test]
    fn the_pivot_period_follows_the_bar_size() {
        assert_eq!(pivot_period(&[0, 30]), PivotPeriod::FifteenMinutes);
        assert_eq!(pivot_period(&[0, 120]), PivotPeriod::Day);
        assert_eq!(pivot_period(&[0, 1800]), PivotPeriod::Week);
        assert_eq!(pivot_period(&[0, DAY, 4 * DAY]), PivotPeriod::Month);
        assert_eq!(pivot_period(&[0, 7 * DAY]), PivotPeriod::Year);
    }

    #[test]
    fn intraday_pivots_use_the_previous_day() {
        // Monday 23 Nov 2026 (day 20 780): three 2-minute bars, then two on Tuesday.
        let monday = 20_780 * DAY + 36_000;
        let tuesday = monday + DAY;
        let store = store_at(
            &[monday, monday + 120, monday + 240, tuesday, tuesday + 120],
            &[
                (10.0, 8.0, 9.0),
                (12.0, 9.0, 11.0),
                (11.0, 7.0, 10.0),
                (20.0, 19.0, 19.5),
                (21.0, 18.0, 20.0),
            ],
        );
        let values = pivot_points_store(&store, &mut HashMap::new());
        let nan = f64::NAN;

        // Monday: high 12, low 7, close 10, so P = 29 / 3. Monday's own bars have no levels.
        let p = 29.0 / 3.0;
        assert_series_close(&values[0].values, &[nan, nan, nan, p, p]);
        assert_series_close(
            &values[1].values,
            &[nan, nan, nan, 2.0 * p - 7.0, 2.0 * p - 7.0],
        );
        assert_series_close(
            &values[2].values,
            &[nan, nan, nan, 2.0 * p - 12.0, 2.0 * p - 12.0],
        );
        assert_series_close(&values[3].values, &[nan, nan, nan, p + 5.0, p + 5.0]);
        assert_series_close(&values[4].values, &[nan, nan, nan, p - 5.0, p - 5.0]);
        assert_series_close(
            &values[5].values,
            &[
                nan,
                nan,
                nan,
                12.0 + 2.0 * (p - 7.0),
                12.0 + 2.0 * (p - 7.0),
            ],
        );
        assert_series_close(
            &values[6].values,
            &[
                nan,
                nan,
                nan,
                7.0 - 2.0 * (12.0 - p),
                7.0 - 2.0 * (12.0 - p),
            ],
        );
        assert_eq!(latest_pivot_points_store(&store)[0], Some(p));
    }

    #[test]
    fn daily_pivots_use_the_previous_month() {
        // 29 and 30 Oct 2026 (days 20 755 and 20 756), then 2 Nov.
        let store = store_at(
            &[20_755 * DAY, 20_756 * DAY, 20_759 * DAY],
            &[(10.0, 4.0, 6.0), (8.0, 5.0, 7.0), (9.0, 6.0, 8.0)],
        );
        let values = pivot_points_store(&store, &mut HashMap::new());
        let nan = f64::NAN;
        // October: high 10, low 4, last close 7.
        assert_series_close(&values[0].values, &[nan, nan, 7.0]);
    }

    #[test]
    fn no_levels_without_a_previous_period() {
        let store = store_at(&[36_000], &[(10.0, 8.0, 9.0)]);
        assert_eq!(latest_pivot_points_store(&store), [None; 7]);
    }
}

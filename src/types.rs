use crate::series::RcSeries;
use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

macro_rules! indicator_kinds {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        #[allow(non_camel_case_types)]
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub(crate) enum IndicatorKind {
            $($variant,)+
        }

        impl IndicatorKind {
            pub(crate) fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }
        }

        impl FromStr for IndicatorKind {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($name => Ok(Self::$variant),)+
                    _ => Err(()),
                }
            }
        }
    };
}

indicator_kinds! {
    SMA => "SMA",
    EMA => "EMA",
    RSI => "RSI",
    STOCH_RSI => "STOCH_RSI",
    CCI => "CCI",
    OBV => "OBV",
    BB => "BB",
    MACD => "MACD",
    ATR => "ATR",
    ADX => "ADX",
    SUPERTREND => "SUPERTREND",
    KELTNER => "KELTNER",
    DONCHIAN => "DONCHIAN",
    PARABOLIC_SAR => "PARABOLIC_SAR",
    ICHIMOKU => "ICHIMOKU",
    PIVOT_POINTS => "PIVOT_POINTS",
    ROC => "ROC",
    AROON => "AROON",
    CMF => "CMF",
    ADL => "ADL",
    WMA => "WMA",
    HMA => "HMA",
    LINEAR_REGRESSION => "LINEAR_REGRESSION",
    TRIX => "TRIX",
    DEMA => "DEMA",
    TEMA => "TEMA",
    TRIMA => "TRIMA",
    STDDEV => "STDDEV",
    ENVELOPE => "ENVELOPE",
    TSI => "TSI",
    KST => "KST",
    BOP => "BOP",
    DPO => "DPO",
    MOMENTUM => "MOMENTUM",
    ULTIMATE_OSCILLATOR => "ULTIMATE_OSCILLATOR",
    CHAIKIN_OSCILLATOR => "CHAIKIN_OSCILLATOR",
    FORCE_INDEX => "FORCE_INDEX",
    VWMA => "VWMA",
    WILLIAMS_AD => "WILLIAMS_AD",
    CHAIKIN_VOLATILITY => "CHAIKIN_VOLATILITY",
    PRICE_CHANNEL => "PRICE_CHANNEL",
    STARC => "STARC",
    VWAP => "VWAP",
    STOCHASTIC => "STOCHASTIC",
    WILLIAMS_R => "WILLIAMS_R",
    MFI => "MFI",
    PPO => "PPO",
    MEDIAN_PRICE => "MEDIAN_PRICE",
    HIGHEST_HIGH => "HIGHEST_HIGH",
    LOWEST_LOW => "LOWEST_LOW",
    ALLIGATOR => "ALLIGATOR",
    ATR_BANDS => "ATR_BANDS",
    HIGH_LOW_BANDS => "HIGH_LOW_BANDS",
    FRACTAL_CHAOS_BANDS => "FRACTAL_CHAOS_BANDS",
    GMMA => "GMMA",
    LINEAR_REG_FORECAST => "LINEAR_REG_FORECAST",
    LINEAR_REG_INTERCEPT => "LINEAR_REG_INTERCEPT",
    ANCHORED_VWAP => "ANCHORED_VWAP",
    TYPICAL_PRICE => "TYPICAL_PRICE",
    WEIGHTED_CLOSE => "WEIGHTED_CLOSE",
    MA_CROSS => "MA_CROSS",
    RAINBOW_MA => "RAINBOW_MA",
    PRIME_NUMBER_BANDS => "PRIME_NUMBER_BANDS",
    TIME_SERIES_FORECAST => "TIME_SERIES_FORECAST",
    VALUATION_LINES => "VALUATION_LINES",
    BETA => "BETA",
    CORRELATION_COEFFICIENT => "CORRELATION_COEFFICIENT",
    PERFORMANCE_INDEX => "PERFORMANCE_INDEX",
    PRICE_RELATIVE => "PRICE_RELATIVE",
    AWESOME_OSCILLATOR => "AWESOME_OSCILLATOR",
    BOLLINGER_PCT_B => "BOLLINGER_PCT_B",
    CENTER_OF_GRAVITY => "CENTER_OF_GRAVITY",
    CHANDE_FORECAST => "CHANDE_FORECAST",
    CHANDE_MOMENTUM => "CHANDE_MOMENTUM",
    COPPOCK_CURVE => "COPPOCK_CURVE",
    DISPARITY_INDEX => "DISPARITY_INDEX",
    EASE_OF_MOVEMENT => "EASE_OF_MOVEMENT",
    EHLER_FISHER => "EHLER_FISHER",
    ELDER_RAY => "ELDER_RAY",
    FRACTAL_CHAOS_OSCILLATOR => "FRACTAL_CHAOS_OSCILLATOR",
    GATOR_OSCILLATOR => "GATOR_OSCILLATOR",
    INTRADAY_MOMENTUM => "INTRADAY_MOMENTUM",
    LINEAR_REG_SLOPE => "LINEAR_REG_SLOPE",
    MA_DEVIATION => "MA_DEVIATION",
    PRETTY_GOOD_OSCILLATOR => "PRETTY_GOOD_OSCILLATOR",
    PRICE_MOMENTUM_OSCILLATOR => "PRICE_MOMENTUM_OSCILLATOR",
    PRICE_OSCILLATOR => "PRICE_OSCILLATOR",
    RAINBOW_OSCILLATOR => "RAINBOW_OSCILLATOR",
    RAVI => "RAVI",
    RELATIVE_VIGOR => "RELATIVE_VIGOR",
    SCHAFF_TREND_CYCLE => "SCHAFF_TREND_CYCLE",
    STOCHASTIC_MOMENTUM => "STOCHASTIC_MOMENTUM",
    SWING_INDEX => "SWING_INDEX",
    TREND_INTENSITY => "TREND_INTENSITY",
    VOLUME_OSCILLATOR => "VOLUME_OSCILLATOR",
    KLINGER_VOLUME => "KLINGER_VOLUME",
    MARKET_FACILITATION => "MARKET_FACILITATION",
    NEGATIVE_VOLUME_INDEX => "NEGATIVE_VOLUME_INDEX",
    POSITIVE_VOLUME_INDEX => "POSITIVE_VOLUME_INDEX",
    PRICE_VOLUME_TREND => "PRICE_VOLUME_TREND",
    TRADE_VOLUME_INDEX => "TRADE_VOLUME_INDEX",
    TWIGGS_MONEY_FLOW => "TWIGGS_MONEY_FLOW",
    PROJECTED_AGGREGATE_VOLUME => "PROJECTED_AGGREGATE_VOLUME",
    PROJECTED_VOLUME_AT_TIME => "PROJECTED_VOLUME_AT_TIME",
    HISTORICAL_VOLATILITY => "HISTORICAL_VOLATILITY",
    LINEAR_REG_R2 => "LINEAR_REG_R2",
    PRIME_NUMBER_OSCILLATOR => "PRIME_NUMBER_OSCILLATOR",
    RANDOM_WALK_INDEX => "RANDOM_WALK_INDEX",
    DARVAS_BOX => "DARVAS_BOX",
    VOLUME_PROFILE => "VOLUME_PROFILE",
    CHOPPINESS_INDEX => "CHOPPINESS_INDEX",
    ELDER_IMPULSE => "ELDER_IMPULSE",
    GONOGO_TREND => "GONOGO_TREND",
    PSYCHOLOGICAL_LINE => "PSYCHOLOGICAL_LINE",
    QSTICK => "QSTICK",
    SHINOHARA_INTENSITY => "SHINOHARA_INTENSITY",
    ULCER_INDEX => "ULCER_INDEX",
    VERTICAL_HORIZONTAL_FILTER => "VERTICAL_HORIZONTAL_FILTER",
    VORTEX_INDICATOR => "VORTEX_INDICATOR",
    ZIGZAG => "ZIGZAG",
    BOLLINGER_BANDWIDTH => "BOLLINGER_BANDWIDTH",
    DONCHIAN_WIDTH => "DONCHIAN_WIDTH",
    GOPALAKRISHNAN_RANGE => "GOPALAKRISHNAN_RANGE",
    HIGH_MINUS_LOW => "HIGH_MINUS_LOW",
    MASS_INDEX => "MASS_INDEX",
    RELATIVE_VOLATILITY => "RELATIVE_VOLATILITY",
    TRUE_RANGE => "TRUE_RANGE",
    VOLUME_CHART => "VOLUME_CHART",
    VOLUME_ROC => "VOLUME_ROC",
    VOLUME_UNDERLAY => "VOLUME_UNDERLAY",
}

impl IndicatorKind {
    pub(crate) fn needs_period(self) -> bool {
        !matches!(
            self,
            Self::OBV
                | Self::ADL
                | Self::VWAP
                | Self::WILLIAMS_AD
                | Self::KST
                | Self::BOP
                | Self::PARABOLIC_SAR
                | Self::ICHIMOKU
                | Self::PIVOT_POINTS
                | Self::MACD
                | Self::PPO
                | Self::MEDIAN_PRICE
                | Self::ALLIGATOR
                | Self::GMMA
                | Self::ANCHORED_VWAP
                | Self::TYPICAL_PRICE
                | Self::WEIGHTED_CLOSE
                | Self::PRIME_NUMBER_BANDS
                | Self::PERFORMANCE_INDEX
                | Self::AWESOME_OSCILLATOR
                | Self::COPPOCK_CURVE
                | Self::FRACTAL_CHAOS_OSCILLATOR
                | Self::FRACTAL_CHAOS_BANDS
                | Self::GATOR_OSCILLATOR
                | Self::KLINGER_VOLUME
                | Self::PRICE_VOLUME_TREND
                | Self::TRADE_VOLUME_INDEX
                | Self::DARVAS_BOX
                | Self::ZIGZAG
                | Self::TRUE_RANGE
                | Self::VOLUME_CHART
                | Self::VOLUME_UNDERLAY
                | Self::MARKET_FACILITATION
                | Self::SWING_INDEX
                | Self::PRIME_NUMBER_OSCILLATOR
                | Self::HIGH_MINUS_LOW
                | Self::VOLUME_OSCILLATOR
                | Self::PRICE_OSCILLATOR
                | Self::SCHAFF_TREND_CYCLE
        )
    }

    pub(crate) fn uses_macd_params(self) -> bool {
        matches!(
            self,
            Self::MACD
                | Self::PPO
                | Self::CHAIKIN_OSCILLATOR
                | Self::MA_CROSS
                | Self::PRICE_OSCILLATOR
                | Self::VOLUME_OSCILLATOR
                | Self::SCHAFF_TREND_CYCLE
        )
    }
}

impl fmt::Display for IndicatorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub(crate) struct Indicator {
    pub id: u32,
    pub kind: IndicatorKind,
    pub period: usize,
    pub stoch_period: usize,
    pub smooth: usize,
    pub signal: usize,
    pub tenkan_period: usize,
    pub kijun_period: usize,
    pub senkou_b_period: usize,
    pub macd: Option<MacdParams>,
    pub multiplier: f64,
    pub psar_step: f64,
    pub psar_max_step: f64,
    pub anchor: usize,
    pub outputs: IndicatorArena,
}

#[derive(Clone, Copy)]
pub(crate) struct MacdParams {
    pub fast: usize,
    pub slow: usize,
    pub signal: usize,
}

#[derive(Deserialize)]
pub(crate) struct IndicatorConfig {
    pub kind: String,
    pub period: Option<usize>,
    pub stoch_period: Option<usize>,
    pub smooth: Option<usize>,
    pub fast: Option<usize>,
    pub slow: Option<usize>,
    pub signal: Option<usize>,
    pub multiplier: Option<f64>,
    pub tenkan_period: Option<usize>,
    pub kijun_period: Option<usize>,
    pub senkou_b_period: Option<usize>,
    pub psar_step: Option<f64>,
    pub psar_max_step: Option<f64>,
    pub anchor: Option<usize>,
}

#[derive(Clone, Serialize)]
pub(crate) struct IndicatorOutput {
    pub name: String,
    pub values: Vec<f64>,
}

#[derive(Clone)]
pub(crate) struct NamedSeries {
    pub name: String,
    pub values: RcSeries,
}

#[allow(dead_code)]
pub(crate) trait NamedOutputLike {
    fn name(&self) -> &str;
    fn values(&self) -> &[f64];
}

impl NamedOutputLike for IndicatorOutput {
    fn name(&self) -> &str {
        &self.name
    }

    fn values(&self) -> &[f64] {
        &self.values
    }
}

impl NamedOutputLike for NamedSeries {
    fn name(&self) -> &str {
        &self.name
    }

    fn values(&self) -> &[f64] {
        self.values.as_slice()
    }
}

/// Per-slot Vec storage for indicator outputs. Each slot is an independent
/// Vec<f64> that can be appended to without copying other slots.
pub(crate) struct IndicatorArena {
    names: Vec<String>,
    values: Vec<Vec<f64>>,
}

impl IndicatorArena {
    /// Create from a Vec<IndicatorOutput>.
    pub(crate) fn from_outputs(outputs: Vec<IndicatorOutput>) -> Self {
        let mut names = Vec::with_capacity(outputs.len());
        let mut values = Vec::with_capacity(outputs.len());
        for output in outputs {
            names.push(output.name);
            values.push(output.values);
        }
        Self { names, values }
    }

    pub(crate) fn from_named_outputs(outputs: Vec<NamedSeries>) -> Self {
        let mut names = Vec::with_capacity(outputs.len());
        let mut values = Vec::with_capacity(outputs.len());
        for output in outputs {
            names.push(output.name);
            values.push(crate::series::rc_into_owned(output.values));
        }
        Self { names, values }
    }

    /// Get the slice for a named output.
    pub(crate) fn get(&self, name: &str) -> Option<&[f64]> {
        let idx = self.names.iter().position(|s| s == name)?;
        Some(&self.values[idx])
    }

    #[inline]
    pub(crate) fn get_slot(&self, slot_idx: usize) -> Option<&[f64]> {
        self.values.get(slot_idx).map(Vec::as_slice)
    }

    #[inline]
    pub(crate) fn value_at_slot(&self, slot_idx: usize, index: usize) -> Option<f64> {
        self.values[slot_idx]
            .get(index)
            .copied()
            .and_then(|v| if v.is_nan() { None } else { Some(v) })
    }

    /// Ensure all slots have at least `target_len` values (pad with NaN).
    #[inline]
    pub(crate) fn ensure_len(&mut self, target_len: usize) {
        for slot in &mut self.values {
            if slot.len() < target_len {
                slot.resize(target_len, f64::NAN);
            }
        }
    }

    /// Set the last value for a named output. If the slot doesn't exist, create it.
    #[inline]
    pub(crate) fn upsert_last(&mut self, name: &str, target_len: usize, value: f64) {
        if let Some(idx) = self.names.iter().position(|s| s == name) {
            let slot = &mut self.values[idx];
            if slot.len() < target_len {
                slot.resize(target_len, f64::NAN);
            }
            let last = slot.len() - 1;
            slot[last] = value;
        } else {
            // New slot — create NaN-filled Vec, set last value
            self.names.push(name.to_string());
            let mut slot = vec![f64::NAN; target_len];
            if target_len > 0 {
                slot[target_len - 1] = value;
            }
            self.values.push(slot);
        }
    }

    /// Resolve a slot name to its index. Returns None if the slot doesn't exist.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn slot_index(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|s| s == name)
    }

    /// Set the last value for a slot by its pre-resolved index. No string lookup.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn upsert_last_at(&mut self, slot_idx: usize, target_len: usize, value: f64) {
        let slot = &mut self.values[slot_idx];
        if slot.len() < target_len {
            slot.resize(target_len, f64::NAN);
        }
        let last = slot.len() - 1;
        slot[last] = value;
    }

    /// Iterate over (name, slice) for all slots.
    pub(crate) fn iter_slots(&self) -> impl Iterator<Item = (&str, &[f64])> {
        self.names
            .iter()
            .zip(self.values.iter())
            .map(|(name, values)| (name.as_str(), values.as_slice()))
    }
}

#[derive(Serialize)]
pub(crate) struct IndicatorLatestValue {
    pub output: String,
    pub value: Option<f64>,
}

#[derive(Default, Serialize)]
pub(crate) struct DagDebug {
    pub nodes: Vec<String>,
    pub edges: Vec<DagEdge>,
}

#[derive(Serialize)]
pub(crate) struct DagEdge {
    pub from: String,
    pub to: String,
}

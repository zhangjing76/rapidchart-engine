use crate::types::{DagEdge, Indicator, MacdParams};

/// Returns true if the indicator kind requires a `period` parameter.
fn needs_period(kind: &str) -> bool {
    // These indicator kinds do NOT accept a period parameter, so skip the period check.
    // This list only contains kinds that receive no period argument in dispatch.rs.
    let no_period = matches!(
        kind,
        "OBV" | "ADL" | "VWAP" | "WILLIAMS_AD" | "KST" | "BOP"
            | "PARABOLIC_SAR" | "ICHIMOKU" | "PIVOT_POINTS" | "MACD" | "PPO"
            | "MEDIAN_PRICE" | "ALLIGATOR" | "GMMA" | "ANCHORED_VWAP"
            | "TYPICAL_PRICE" | "WEIGHTED_CLOSE" | "PRIME_NUMBER_BANDS"
            | "PERFORMANCE_INDEX" | "AWESOME_OSCILLATOR" | "COPPOCK_CURVE"
            | "FRACTAL_CHAOS_OSCILLATOR" | "FRACTAL_CHAOS_BANDS"
            | "GATOR_OSCILLATOR" | "KLINGER_VOLUME" | "PRICE_VOLUME_TREND"
            | "TRADE_VOLUME_INDEX" | "DARVAS_BOX" | "ZIGZAG" | "TRUE_RANGE"
            | "VOLUME_CHART" | "VOLUME_UNDERLAY" | "MARKET_FACILITATION"
            | "SWING_INDEX" | "PRIME_NUMBER_OSCILLATOR" | "HIGH_MINUS_LOW"
            | "VOLUME_OSCILLATOR" | "PRICE_OSCILLATOR" | "SCHAFF_TREND_CYCLE"
    );
    !no_period
}

pub(crate) fn is_valid_kind(kind: &str) -> bool {
    matches!(
        kind,
        "SMA"
            | "EMA"
            | "RSI"
            | "STOCH_RSI"
            | "CCI"
            | "OBV"
            | "BB"
            | "MACD"
            | "ATR"
            | "ADX"
            | "SUPERTREND"
            | "KELTNER"
            | "DONCHIAN"
            | "PARABOLIC_SAR"
            | "ICHIMOKU"
            | "PIVOT_POINTS"
            | "ROC"
            | "AROON"
            | "CMF"
            | "ADL"
            | "WMA"
            | "HMA"
            | "LINEAR_REGRESSION"
            | "TRIX"
            | "DEMA"
            | "TEMA"
            | "TRIMA"
            | "STDDEV"
            | "ENVELOPE"
            | "TSI"
            | "KST"
            | "BOP"
            | "DPO"
            | "MOMENTUM"
            | "ULTIMATE_OSCILLATOR"
            | "CHAIKIN_OSCILLATOR"
            | "FORCE_INDEX"
            | "VWMA"
            | "WILLIAMS_AD"
            | "CHAIKIN_VOLATILITY"
            | "PRICE_CHANNEL"
            | "STARC"
            | "VWAP"
            | "STOCHASTIC"
            | "WILLIAMS_R"
            | "MFI"
            | "PPO"
            | "MEDIAN_PRICE"
            | "HIGHEST_HIGH"
            | "LOWEST_LOW"
            | "ALLIGATOR"
            | "ATR_BANDS"
            | "HIGH_LOW_BANDS"
            | "FRACTAL_CHAOS_BANDS"
            | "GMMA"
            | "LINEAR_REG_FORECAST"
            | "LINEAR_REG_INTERCEPT"
            | "ANCHORED_VWAP"
            | "TYPICAL_PRICE"
            | "WEIGHTED_CLOSE"
            | "MA_CROSS"
            | "RAINBOW_MA"
            | "PRIME_NUMBER_BANDS"
            | "TIME_SERIES_FORECAST"
            | "VALUATION_LINES"
            | "BETA"
            | "CORRELATION_COEFFICIENT"
            | "PERFORMANCE_INDEX"
            | "PRICE_RELATIVE"
            | "AWESOME_OSCILLATOR"
            | "BOLLINGER_PCT_B"
            | "CENTER_OF_GRAVITY"
            | "CHANDE_FORECAST"
            | "CHANDE_MOMENTUM"
            | "COPPOCK_CURVE"
            | "DISPARITY_INDEX"
            | "EASE_OF_MOVEMENT"
            | "EHLER_FISHER"
            | "ELDER_RAY"
            | "FRACTAL_CHAOS_OSCILLATOR"
            | "GATOR_OSCILLATOR"
            | "INTRADAY_MOMENTUM"
            | "LINEAR_REG_SLOPE"
            | "MA_DEVIATION"
            | "PRETTY_GOOD_OSCILLATOR"
            | "PRICE_MOMENTUM_OSCILLATOR"
            | "PRICE_OSCILLATOR"
            | "RAINBOW_OSCILLATOR"
            | "RAVI"
            | "RELATIVE_VIGOR"
            | "SCHAFF_TREND_CYCLE"
            | "STOCHASTIC_MOMENTUM"
            | "SWING_INDEX"
            | "TREND_INTENSITY"
            | "VOLUME_OSCILLATOR"
            | "KLINGER_VOLUME"
            | "MARKET_FACILITATION"
            | "NEGATIVE_VOLUME_INDEX"
            | "POSITIVE_VOLUME_INDEX"
            | "PRICE_VOLUME_TREND"
            | "TRADE_VOLUME_INDEX"
            | "TWIGGS_MONEY_FLOW"
            | "PROJECTED_AGGREGATE_VOLUME"
            | "PROJECTED_VOLUME_AT_TIME"
            | "HISTORICAL_VOLATILITY"
            | "LINEAR_REG_R2"
            | "PRIME_NUMBER_OSCILLATOR"
            | "RANDOM_WALK_INDEX"
            | "DARVAS_BOX"
            | "VOLUME_PROFILE"
            | "CHOPPINESS_INDEX"
            | "ELDER_IMPULSE"
            | "GONOGO_TREND"
            | "PSYCHOLOGICAL_LINE"
            | "QSTICK"
            | "SHINOHARA_INTENSITY"
            | "ULCER_INDEX"
            | "VERTICAL_HORIZONTAL_FILTER"
            | "VORTEX_INDICATOR"
            | "ZIGZAG"
            | "BOLLINGER_BANDWIDTH"
            | "DONCHIAN_WIDTH"
            | "GOPALAKRISHNAN_RANGE"
            | "HIGH_MINUS_LOW"
            | "MASS_INDEX"
            | "RELATIVE_VOLATILITY"
            | "TRUE_RANGE"
            | "VOLUME_CHART"
            | "VOLUME_ROC"
            | "VOLUME_UNDERLAY"
    )
}

pub(crate) fn is_visible_output(name: &str) -> bool {
    !matches!(
        name,
        "fast_ema"
            | "slow_ema"
            | "avg_gain"
            | "avg_loss"
            | "tr_avg"
            | "plus_dm_avg"
            | "minus_dm_avg"
            | "dx"
            | "upper_band"
            | "lower_band"
            | "trend"
            | "ep"
            | "af"
            | "cumulative_pv"
            | "cumulative_volume"
            | "atr_state"
            | "ema1"
            | "ema2"
            | "ema3"
            | "m_ema1"
            | "m_ema2"
            | "a_ema1"
            | "a_ema2"
            | "fi_ema"
            | "adl"
            | "hl_ema"
    )
}

pub(crate) fn indicator_node(indicator: &Indicator) -> String {
    match (indicator.kind.as_str(), indicator.macd) {
        ("MACD", Some(macd)) | ("PPO", Some(macd)) => format!(
            "{}({},{},{})#{}",
            indicator.kind, macd.fast, macd.slow, macd.signal, indicator.id
        ),
        ("CHAIKIN_OSCILLATOR", Some(macd)) => format!(
            "CHAIKIN_OSCILLATOR({},{})#{}",
            macd.fast, macd.slow, indicator.id
        ),
        _ => format!("{}#{}", indicator.kind, indicator.id),
    }
}

pub(crate) fn edge(from: &str, to: &str) -> DagEdge {
    DagEdge {
        from: from.to_string(),
        to: to.to_string(),
    }
}

pub(crate) fn indicator_nodes(indicator: &Indicator) -> Vec<String> {
    match indicator.kind.as_str() {
        "SMA" => vec![format!("sma:close:{}", indicator.period)],
        "EMA" => vec![format!("ema:close:{}", indicator.period)],
        "WMA" => vec![format!("wma:close:{}", indicator.period)],
        "HMA" => vec![
            format!("wma:close:{}", indicator.period / 2),
            format!("wma:close:{}", indicator.period),
            format!("hma:close:{}", indicator.period),
        ],
        "LINEAR_REGRESSION" => vec![format!("linreg:close:{}", indicator.period)],
        "DEMA" => vec![
            format!("ema:close:{}", indicator.period),
            format!("dema:ema2:{}", indicator.period),
            format!("dema:value:{}", indicator.period),
        ],
        "TEMA" => vec![
            format!("ema:close:{}", indicator.period),
            format!("tema:ema2:{}", indicator.period),
            format!("tema:ema3:{}", indicator.period),
            format!("tema:value:{}", indicator.period),
        ],
        "TRIMA" => vec![
            format!("sma:close:{}", indicator.period),
            format!("trima:value:{}", indicator.period),
        ],
        "STDDEV" => vec![format!("stddev:close:{}", indicator.period)],
        "ENVELOPE" => vec![
            format!("sma:close:{}", indicator.period),
            format!(
                "envelope:upper:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "envelope:middle:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "envelope:lower:{}:{}",
                indicator.period, indicator.multiplier
            ),
        ],
        "TRIX" => vec![
            format!("ema:close:{}", indicator.period),
            format!("trix:ema2:{}", indicator.period),
            format!("trix:value:{}", indicator.period),
        ],
        "TSI" => vec![format!(
            "tsi:{}:{}",
            indicator.period, indicator.stoch_period
        )],
        "DPO" => vec![
            format!("sma:close:{}", indicator.period),
            format!("dpo:close:{}", indicator.period),
        ],
        "MOMENTUM" => vec![format!("momentum:close:{}", indicator.period)],
        "RSI" => vec![format!("rsi:close:{}", indicator.period)],
        "ROC" => vec![format!("roc:close:{}", indicator.period)],
        "CCI" => vec![format!("cci:hlc:{}", indicator.period)],
        "CMF" => vec![format!("cmf:hlcv:{}", indicator.period)],
        "MFI" => vec![format!("mfi:hlcv:{}", indicator.period)],
        "WILLIAMS_R" => vec![format!("willr:hlc:{}", indicator.period)],
        "VWMA" => vec![format!("vwma:close:volume:{}", indicator.period)],
        "WILLIAMS_AD" => vec!["wad:ohlc".to_string()],
        "CHAIKIN_VOLATILITY" => vec![
            format!("cvol:ema:{}", indicator.period),
            format!("cvol:value:{}", indicator.period),
        ],
        "PRICE_CHANNEL" => vec![
            format!("price_channel:upper:{}", indicator.period),
            format!("price_channel:middle:{}", indicator.period),
            format!("price_channel:lower:{}", indicator.period),
        ],
        "STARC" => vec![
            format!("sma:close:{}", indicator.period),
            format!("atr:ohlc:{}", indicator.period),
            format!("starc:upper:{}:{}", indicator.period, indicator.multiplier),
            format!("starc:middle:{}:{}", indicator.period, indicator.multiplier),
            format!("starc:lower:{}:{}", indicator.period, indicator.multiplier),
        ],
        "PARABOLIC_SAR" => vec![format!(
            "psar:ohlc:{}:{}",
            indicator.psar_step, indicator.psar_max_step
        )],
        "ICHIMOKU" => vec![
            format!("ichimoku:tenkan:{}", indicator.tenkan_period),
            format!("ichimoku:kijun:{}", indicator.kijun_period),
            format!(
                "ichimoku:senkou_a:{}:{}",
                indicator.tenkan_period, indicator.kijun_period
            ),
            format!("ichimoku:senkou_b:{}", indicator.senkou_b_period),
            "ichimoku:chikou".to_string(),
        ],
        "PIVOT_POINTS" => vec![
            "pivot:pp".to_string(),
            "pivot:r1".to_string(),
            "pivot:s1".to_string(),
            "pivot:r2".to_string(),
            "pivot:s2".to_string(),
        ],
        "AROON" => vec![format!("aroon:hl:{}", indicator.period)],
        "ADL" => vec!["adl:hlcv".to_string()],
        "KST" => vec![
            "roc:close:10".to_string(),
            "roc:close:15".to_string(),
            "roc:close:20".to_string(),
            "roc:close:30".to_string(),
            "kst:value".to_string(),
        ],
        "BOP" => vec!["bop:ohlc".to_string()],
        "ULTIMATE_OSCILLATOR" => vec![format!(
            "uo:{}:{}:{}",
            indicator.period, indicator.stoch_period, indicator.smooth
        )],
        "CHAIKIN_OSCILLATOR" => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            });
            vec![
                "adl:hlcv".to_string(),
                format!("chaikin:{}:{}", p.fast, p.slow),
            ]
        }
        "FORCE_INDEX" => vec![format!("force:close:volume:{}", indicator.period)],
        "SUPERTREND" => vec![
            format!("atr:ohlc:{}", indicator.period),
            format!("supertrend:{}:{}", indicator.period, indicator.multiplier),
        ],
        "KELTNER" => vec![
            format!("ema:close:{}", indicator.period),
            format!("atr:ohlc:{}", indicator.period),
            format!(
                "keltner:upper:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "keltner:middle:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "keltner:lower:{}:{}",
                indicator.period, indicator.multiplier
            ),
        ],
        "DONCHIAN" => vec![
            format!("donchian:upper:{}", indicator.period),
            format!("donchian:middle:{}", indicator.period),
            format!("donchian:lower:{}", indicator.period),
        ],
        "STOCH_RSI" => vec![
            format!("rsi:close:{}", indicator.period),
            format!(
                "stoch:rsi:{}:{}:{}:{}",
                indicator.period, indicator.stoch_period, indicator.smooth, indicator.signal
            ),
        ],
        "ATR" => vec![format!("atr:ohlc:{}", indicator.period)],
        "ADX" => vec![format!("adx:ohlc:{}", indicator.period)],
        "VWAP" => vec!["vwap:hlcv".to_string()],
        "STOCHASTIC" => vec![format!(
            "stoch:hlc:{}:{}",
            indicator.period, indicator.smooth
        )],
        "BB" => vec![
            format!("sma:close:{}", indicator.period),
            format!("bb:upper:{}:{}", indicator.period, indicator.multiplier),
            format!("bb:middle:{}:{}", indicator.period, indicator.multiplier),
            format!("bb:lower:{}:{}", indicator.period, indicator.multiplier),
        ],
        "OBV" => vec!["obv:close:volume".to_string()],
        "MACD" => {
            let m = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            vec![
                format!("ema:close:{}", m.fast),
                format!("ema:close:{}", m.slow),
            ]
        }
        "PPO" => {
            let m = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            vec![
                format!("ema:close:{}", m.fast),
                format!("ema:close:{}", m.slow),
                format!("ppo:{}:{}:{}", m.fast, m.slow, m.signal),
            ]
        }
        "MEDIAN_PRICE" => vec!["median_price:hl".to_string()],
        "HIGHEST_HIGH" => vec![format!("highest_high:h:{}", indicator.period)],
        "LOWEST_LOW" => vec![format!("lowest_low:l:{}", indicator.period)],
        "ALLIGATOR" => vec![
            "alligator:jaw".to_string(),
            "alligator:teeth".to_string(),
            "alligator:lips".to_string(),
        ],
        "ATR_BANDS" => vec![
            format!("ema:close:{}", indicator.period),
            format!("atr:ohlc:{}", indicator.period),
            format!("atr_bands:upper:{}:{}", indicator.period, indicator.multiplier),
            format!("atr_bands:lower:{}:{}", indicator.period, indicator.multiplier),
        ],
        "HIGH_LOW_BANDS" => vec![
            format!("hlbands:upper:{}", indicator.period),
            format!("hlbands:middle:{}", indicator.period),
            format!("hlbands:lower:{}", indicator.period),
        ],
        "FRACTAL_CHAOS_BANDS" => vec![
            "fcb:upper".to_string(),
            "fcb:lower".to_string(),
        ],
        "GMMA" => {
            let mut nodes = Vec::with_capacity(12);
            for &p in &[3, 5, 8, 10, 12, 15, 30, 35, 40, 45, 50, 60] {
                nodes.push(format!("ema:close:{p}"));
            }
            nodes
        }
        "LINEAR_REG_FORECAST" => vec![format!("linreg_forecast:close:{}", indicator.period)],
        "LINEAR_REG_INTERCEPT" => vec![format!("linreg_intercept:close:{}", indicator.period)],
        "ANCHORED_VWAP" => vec![format!("anchored_vwap:{}", indicator.anchor)],
        "TYPICAL_PRICE" => vec!["typical_price:hlc".to_string()],
        "WEIGHTED_CLOSE" => vec!["weighted_close:hlc".to_string()],
        "MA_CROSS" => {
            let p = indicator.macd.unwrap_or(MacdParams { fast: indicator.period, slow: indicator.stoch_period, signal: 9 });
            vec![
                format!("sma:close:{}", p.fast),
                format!("sma:close:{}", p.slow),
            ]
        }
        "RAINBOW_MA" => (1..=10).map(|i| format!("rainbow:r{}:{}", i, indicator.period)).collect(),
        "PRIME_NUMBER_BANDS" => vec!["pnb:upper".to_string(), "pnb:lower".to_string()],
        "TIME_SERIES_FORECAST" => vec![format!("linreg_forecast:close:{}", indicator.period)],
        "VALUATION_LINES" => vec![
            format!("sma:close:{}", indicator.period),
            format!("valuation:upper:{}:{}", indicator.period, indicator.multiplier),
            format!("valuation:lower:{}:{}", indicator.period, indicator.multiplier),
        ],
        "BETA" => vec![format!("beta:close:{}", indicator.period)],
        "CORRELATION_COEFFICIENT" => vec![format!("correl:close:{}", indicator.period)],
        "PERFORMANCE_INDEX" => vec!["perf_index:close".to_string()],
        "PRICE_RELATIVE" => vec![format!("price_relative:close:{}", indicator.period)],
        "AWESOME_OSCILLATOR" => vec!["ao:hl".to_string()],
        "BOLLINGER_PCT_B" => vec![format!("bb_pctb:{}:{}", indicator.period, indicator.multiplier)],
        "CENTER_OF_GRAVITY" => vec![format!("cog:close:{}", indicator.period)],
        "CHANDE_FORECAST" => vec![format!("cfo:close:{}", indicator.period)],
        "CHANDE_MOMENTUM" => vec![format!("cmo:close:{}", indicator.period)],
        "COPPOCK_CURVE" => vec!["coppock:close".to_string()],
        "DISPARITY_INDEX" => vec![
            format!("ema:close:{}", indicator.period),
            format!("disparity:close:{}", indicator.period),
        ],
        "EASE_OF_MOVEMENT" => vec![format!("emv:hlv:{}", indicator.period)],
        "EHLER_FISHER" => vec![format!("fisher:hl:{}", indicator.period)],
        "ELDER_RAY" => vec![
            format!("ema:close:{}", indicator.period),
            format!("elder_ray:hl:{}", indicator.period),
        ],
        "FRACTAL_CHAOS_OSCILLATOR" => vec!["fco:hl".to_string()],
        "GATOR_OSCILLATOR" => vec!["gator:upper".to_string(), "gator:lower".to_string()],
        "INTRADAY_MOMENTUM" => vec![format!("imi:oc:{}", indicator.period)],
        "LINEAR_REG_SLOPE" => vec![format!("linreg_slope:close:{}", indicator.period)],
        "MA_DEVIATION" => vec![format!("sma:close:{}", indicator.period), format!("ma_dev:close:{}", indicator.period)],
        "PRETTY_GOOD_OSCILLATOR" => vec![
            format!("sma:close:{}", indicator.period),
            format!("atr:ohlc:{}", indicator.period),
            format!("pgo:close:{}", indicator.period),
        ],
        "PRICE_MOMENTUM_OSCILLATOR" => vec![format!("pmo:close:{}:{}", indicator.period, indicator.smooth)],
        "PRICE_OSCILLATOR" => {
            let p = indicator.macd.unwrap_or(MacdParams { fast: 12, slow: 26, signal: 9 });
            vec![format!("price_osc:close:{}:{}", p.fast, p.slow)]
        }
        "RAINBOW_OSCILLATOR" => vec![format!("rainbow_osc:close:{}", indicator.period)],
        "RAVI" => vec![format!("ravi:close:{}:{}", indicator.period, indicator.stoch_period)],
        "RELATIVE_VIGOR" => vec![format!("rvi:ohlc:{}", indicator.period)],
        "SCHAFF_TREND_CYCLE" => {
            let p = indicator.macd.unwrap_or(MacdParams { fast: 12, slow: 26, signal: 9 });
            vec![format!("stc:{}:{}:{}", p.fast, p.slow, indicator.stoch_period)]
        }
        "STOCHASTIC_MOMENTUM" => vec![format!("smi:hlc:{}:{}", indicator.period, indicator.smooth)],
        "SWING_INDEX" => vec!["swing_index:ohlc".to_string()],
        "TREND_INTENSITY" => vec![format!("sma:close:{}", indicator.period), format!("tii:close:{}", indicator.period)],
        "VOLUME_OSCILLATOR" => {
            let p = indicator.macd.unwrap_or(MacdParams { fast: 5, slow: 10, signal: 9 });
            vec![format!("vol_osc:volume:{}:{}", p.fast, p.slow)]
        }
        "KLINGER_VOLUME" => vec!["klinger:hlcv".to_string()],
        "MARKET_FACILITATION" => vec!["mfi_bw:hlv".to_string()],
        "NEGATIVE_VOLUME_INDEX" => vec!["nvi:cv".to_string()],
        "POSITIVE_VOLUME_INDEX" => vec!["pvi:cv".to_string()],
        "PRICE_VOLUME_TREND" => vec!["pvt:cv".to_string()],
        "TRADE_VOLUME_INDEX" => vec!["tvi:cv".to_string()],
        "TWIGGS_MONEY_FLOW" => vec![format!("tmf:hlcv:{}", indicator.period)],
        "PROJECTED_AGGREGATE_VOLUME" => vec![format!("pav:v:{}", indicator.period)],
        "PROJECTED_VOLUME_AT_TIME" => vec![format!("pvat:v:{}", indicator.period)],
        "HISTORICAL_VOLATILITY" => vec![format!("hv:close:{}", indicator.period)],
        "LINEAR_REG_R2" => vec![format!("linreg_r2:close:{}", indicator.period)],
        "PRIME_NUMBER_OSCILLATOR" => vec!["pno:close".to_string()],
        "RANDOM_WALK_INDEX" => vec![format!("rwi:hlc:{}", indicator.period)],
        "DARVAS_BOX" => vec!["darvas:top".to_string(), "darvas:bottom".to_string()],
        "VOLUME_PROFILE" => vec![format!("vol_profile:hlv:{}", indicator.period)],
        "CHOPPINESS_INDEX" => vec![format!("chop:hlc:{}", indicator.period)],
        "ELDER_IMPULSE" => vec![format!("impulse:close:{}", indicator.period)],
        "GONOGO_TREND" => vec![format!("gonogo:close:{}", indicator.period)],
        "PSYCHOLOGICAL_LINE" => vec![format!("psy:close:{}", indicator.period)],
        "QSTICK" => vec![format!("qstick:oc:{}", indicator.period)],
        "SHINOHARA_INTENSITY" => vec![format!("shinohara:hlc:{}", indicator.period)],
        "ULCER_INDEX" => vec![format!("ulcer:close:{}", indicator.period)],
        "VERTICAL_HORIZONTAL_FILTER" => vec![format!("vhf:close:{}", indicator.period)],
        "VORTEX_INDICATOR" => vec![format!("vortex:hlc:{}", indicator.period)],
        "ZIGZAG" => vec![format!("zigzag:hl:{}", indicator.multiplier)],
        "BOLLINGER_BANDWIDTH" => vec![format!("bb_bw:{}:{}", indicator.period, indicator.multiplier)],
        "DONCHIAN_WIDTH" => vec![format!("donchian_width:{}", indicator.period)],
        "GOPALAKRISHNAN_RANGE" => vec![format!("gapo:hl:{}", indicator.period)],
        "HIGH_MINUS_LOW" => vec!["hml:hl".to_string()],
        "MASS_INDEX" => vec![format!("mass:hl:{}", indicator.period)],
        "RELATIVE_VOLATILITY" => vec![format!("rvi_vol:close:{}", indicator.period)],
        "TRUE_RANGE" => vec!["true_range:hlc".to_string()],
        "VOLUME_CHART" => vec!["vol_chart:v".to_string()],
        "VOLUME_ROC" => vec![format!("vol_roc:v:{}", indicator.period)],
        "VOLUME_UNDERLAY" => vec!["vol_underlay:cv".to_string()],
        _ => vec!["close".to_string()],
    }
}

pub(crate) fn indicator_edges(indicator: &Indicator, indicator_node: &str) -> Vec<DagEdge> {
    match indicator.kind.as_str() {
        "BB" => {
            let sma = format!("sma:close:{}", indicator.period);
            let upper = format!("bb:upper:{}:{}", indicator.period, indicator.multiplier);
            let middle = format!("bb:middle:{}:{}", indicator.period, indicator.multiplier);
            let lower = format!("bb:lower:{}:{}", indicator.period, indicator.multiplier);
            vec![
                edge("close", &sma),
                edge(&sma, &upper),
                edge(&sma, &middle),
                edge(&sma, &lower),
                edge(&upper, indicator_node),
                edge(&middle, indicator_node),
                edge(&lower, indicator_node),
            ]
        }
        "OBV" => vec![
            edge("close", "obv:close:volume"),
            edge("volume", "obv:close:volume"),
            edge("obv:close:volume", indicator_node),
        ],
        "ATR" => {
            let a = format!("atr:ohlc:{}", indicator.period);
            vec![
                edge("high", &a),
                edge("low", &a),
                edge("close", &a),
                edge(&a, indicator_node),
            ]
        }
        "CCI" => {
            let c = format!("cci:hlc:{}", indicator.period);
            vec![
                edge("high", &c),
                edge("low", &c),
                edge("close", &c),
                edge(&c, indicator_node),
            ]
        }
        "VWMA" => {
            let v = format!("vwma:close:volume:{}", indicator.period);
            vec![
                edge("close", &v),
                edge("volume", &v),
                edge(&v, indicator_node),
            ]
        }
        "WILLIAMS_AD" => vec![
            edge("high", "wad:ohlc"),
            edge("low", "wad:ohlc"),
            edge("close", "wad:ohlc"),
            edge("wad:ohlc", indicator_node),
        ],
        "CHAIKIN_VOLATILITY" => {
            let ema = format!("cvol:ema:{}", indicator.period);
            let val = format!("cvol:value:{}", indicator.period);
            vec![
                edge("high", &ema),
                edge("low", &ema),
                edge(&ema, &val),
                edge(&val, indicator_node),
            ]
        }
        "PRICE_CHANNEL" => {
            let u = format!("price_channel:upper:{}", indicator.period);
            let m = format!("price_channel:middle:{}", indicator.period);
            let l = format!("price_channel:lower:{}", indicator.period);
            vec![
                edge("high", &u),
                edge("low", &u),
                edge("high", &m),
                edge("low", &m),
                edge("high", &l),
                edge("low", &l),
                edge(&u, indicator_node),
                edge(&m, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        "STARC" => {
            let sma = format!("sma:close:{}", indicator.period);
            let atr = format!("atr:ohlc:{}", indicator.period);
            let u = format!("starc:upper:{}:{}", indicator.period, indicator.multiplier);
            let m = format!("starc:middle:{}:{}", indicator.period, indicator.multiplier);
            let l = format!("starc:lower:{}:{}", indicator.period, indicator.multiplier);
            vec![
                edge("close", &sma),
                edge("high", &atr),
                edge("low", &atr),
                edge("close", &atr),
                edge(&sma, &u),
                edge(&atr, &u),
                edge(&sma, &m),
                edge(&sma, &l),
                edge(&atr, &l),
                edge(&u, indicator_node),
                edge(&m, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        "DEMA" => {
            let e1 = format!("ema:close:{}", indicator.period);
            let e2 = format!("dema:ema2:{}", indicator.period);
            let d = format!("dema:value:{}", indicator.period);
            vec![
                edge("close", &e1),
                edge(&e1, &e2),
                edge(&e2, &d),
                edge(&d, indicator_node),
            ]
        }
        "TEMA" => {
            let e1 = format!("ema:close:{}", indicator.period);
            let e2 = format!("tema:ema2:{}", indicator.period);
            let e3 = format!("tema:ema3:{}", indicator.period);
            let t = format!("tema:value:{}", indicator.period);
            vec![
                edge("close", &e1),
                edge(&e1, &e2),
                edge(&e2, &e3),
                edge(&e3, &t),
                edge(&t, indicator_node),
            ]
        }
        "TRIMA" => {
            let s = format!("sma:close:{}", indicator.period);
            let t = format!("trima:value:{}", indicator.period);
            vec![edge("close", &s), edge(&s, &t), edge(&t, indicator_node)]
        }
        "STDDEV" => {
            let s = format!("stddev:close:{}", indicator.period);
            vec![edge("close", &s), edge(&s, indicator_node)]
        }
        "ENVELOPE" => {
            let s = format!("sma:close:{}", indicator.period);
            let u = format!(
                "envelope:upper:{}:{}",
                indicator.period, indicator.multiplier
            );
            let m = format!(
                "envelope:middle:{}:{}",
                indicator.period, indicator.multiplier
            );
            let l = format!(
                "envelope:lower:{}:{}",
                indicator.period, indicator.multiplier
            );
            vec![
                edge("close", &s),
                edge(&s, &u),
                edge(&s, &m),
                edge(&s, &l),
                edge(&u, indicator_node),
                edge(&m, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        "WMA" => {
            let w = format!("wma:close:{}", indicator.period);
            vec![edge("close", &w), edge(&w, indicator_node)]
        }
        "HMA" => {
            let h = format!("wma:close:{}", indicator.period / 2);
            let f = format!("wma:close:{}", indicator.period);
            let hma = format!("hma:close:{}", indicator.period);
            vec![
                edge("close", &h),
                edge("close", &f),
                edge(&h, &hma),
                edge(&f, &hma),
                edge(&hma, indicator_node),
            ]
        }
        "LINEAR_REGRESSION" => {
            let l = format!("linreg:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        "TRIX" => {
            let e1 = format!("ema:close:{}", indicator.period);
            let e2 = format!("trix:ema2:{}", indicator.period);
            let t = format!("trix:value:{}", indicator.period);
            vec![
                edge("close", &e1),
                edge(&e1, &e2),
                edge(&e2, &t),
                edge(&t, indicator_node),
            ]
        }
        "TSI" => {
            let t = format!("tsi:{}:{}", indicator.period, indicator.stoch_period);
            vec![edge("close", &t), edge(&t, indicator_node)]
        }
        "DPO" => {
            let s = format!("sma:close:{}", indicator.period);
            let d = format!("dpo:close:{}", indicator.period);
            vec![
                edge("close", &s),
                edge("close", &d),
                edge(&s, &d),
                edge(&d, indicator_node),
            ]
        }
        "MOMENTUM" => {
            let m = format!("momentum:close:{}", indicator.period);
            vec![edge("close", &m), edge(&m, indicator_node)]
        }
        "ROC" => {
            let r = format!("roc:close:{}", indicator.period);
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        "WILLIAMS_R" => {
            let w = format!("willr:hlc:{}", indicator.period);
            vec![
                edge("high", &w),
                edge("low", &w),
                edge("close", &w),
                edge(&w, indicator_node),
            ]
        }
        "MFI" => {
            let m = format!("mfi:hlcv:{}", indicator.period);
            vec![
                edge("high", &m),
                edge("low", &m),
                edge("close", &m),
                edge("volume", &m),
                edge(&m, indicator_node),
            ]
        }
        "CMF" => {
            let c = format!("cmf:hlcv:{}", indicator.period);
            vec![
                edge("high", &c),
                edge("low", &c),
                edge("close", &c),
                edge("volume", &c),
                edge(&c, indicator_node),
            ]
        }
        "SUPERTREND" => {
            let a = format!("atr:ohlc:{}", indicator.period);
            let s = format!("supertrend:{}:{}", indicator.period, indicator.multiplier);
            vec![
                edge("high", &a),
                edge("low", &a),
                edge("close", &a),
                edge("high", &s),
                edge("low", &s),
                edge("close", &s),
                edge(&a, &s),
                edge(&s, indicator_node),
            ]
        }
        "PARABOLIC_SAR" => {
            let p = format!(
                "psar:ohlc:{}:{}",
                indicator.psar_step, indicator.psar_max_step
            );
            vec![
                edge("high", &p),
                edge("low", &p),
                edge("close", &p),
                edge(&p, indicator_node),
            ]
        }
        "ICHIMOKU" => {
            let tk = format!("ichimoku:tenkan:{}", indicator.tenkan_period);
            let kj = format!("ichimoku:kijun:{}", indicator.kijun_period);
            let sa = format!(
                "ichimoku:senkou_a:{}:{}",
                indicator.tenkan_period, indicator.kijun_period
            );
            let sb = format!("ichimoku:senkou_b:{}", indicator.senkou_b_period);
            vec![
                edge("high", &tk),
                edge("low", &tk),
                edge("high", &kj),
                edge("low", &kj),
                edge("high", &sb),
                edge("low", &sb),
                edge("close", "ichimoku:chikou"),
                edge(&tk, &sa),
                edge(&kj, &sa),
                edge(&tk, indicator_node),
                edge(&kj, indicator_node),
                edge(&sa, indicator_node),
                edge(&sb, indicator_node),
                edge("ichimoku:chikou", indicator_node),
            ]
        }
        "PIVOT_POINTS" => vec![
            edge("high", "pivot:pp"),
            edge("low", "pivot:pp"),
            edge("close", "pivot:pp"),
            edge("pivot:pp", "pivot:r1"),
            edge("pivot:pp", "pivot:s1"),
            edge("pivot:pp", "pivot:r2"),
            edge("pivot:pp", "pivot:s2"),
            edge("pivot:pp", indicator_node),
            edge("pivot:r1", indicator_node),
            edge("pivot:s1", indicator_node),
            edge("pivot:r2", indicator_node),
            edge("pivot:s2", indicator_node),
        ],
        "AROON" => {
            let a = format!("aroon:hl:{}", indicator.period);
            vec![edge("high", &a), edge("low", &a), edge(&a, indicator_node)]
        }
        "ADL" => vec![
            edge("high", "adl:hlcv"),
            edge("low", "adl:hlcv"),
            edge("close", "adl:hlcv"),
            edge("volume", "adl:hlcv"),
            edge("adl:hlcv", indicator_node),
        ],
        "KST" => vec![
            edge("close", "roc:close:10"),
            edge("close", "roc:close:15"),
            edge("close", "roc:close:20"),
            edge("close", "roc:close:30"),
            edge("roc:close:10", "kst:value"),
            edge("roc:close:15", "kst:value"),
            edge("roc:close:20", "kst:value"),
            edge("roc:close:30", "kst:value"),
            edge("kst:value", indicator_node),
        ],
        "BOP" => vec![
            edge("high", "bop:ohlc"),
            edge("low", "bop:ohlc"),
            edge("close", "bop:ohlc"),
            edge("bop:ohlc", indicator_node),
        ],
        "ULTIMATE_OSCILLATOR" => {
            let u = format!(
                "uo:{}:{}:{}",
                indicator.period, indicator.stoch_period, indicator.smooth
            );
            vec![
                edge("high", &u),
                edge("low", &u),
                edge("close", &u),
                edge(&u, indicator_node),
            ]
        }
        "CHAIKIN_OSCILLATOR" => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            });
            let c = format!("chaikin:{}:{}", p.fast, p.slow);
            vec![
                edge("high", "adl:hlcv"),
                edge("low", "adl:hlcv"),
                edge("close", "adl:hlcv"),
                edge("volume", "adl:hlcv"),
                edge("adl:hlcv", &c),
                edge(&c, indicator_node),
            ]
        }
        "FORCE_INDEX" => {
            let f = format!("force:close:volume:{}", indicator.period);
            vec![
                edge("close", &f),
                edge("volume", &f),
                edge(&f, indicator_node),
            ]
        }
        "KELTNER" => {
            let ema = format!("ema:close:{}", indicator.period);
            let atr = format!("atr:ohlc:{}", indicator.period);
            let u = format!(
                "keltner:upper:{}:{}",
                indicator.period, indicator.multiplier
            );
            let m = format!(
                "keltner:middle:{}:{}",
                indicator.period, indicator.multiplier
            );
            let l = format!(
                "keltner:lower:{}:{}",
                indicator.period, indicator.multiplier
            );
            vec![
                edge("close", &ema),
                edge("high", &atr),
                edge("low", &atr),
                edge("close", &atr),
                edge(&ema, &u),
                edge(&atr, &u),
                edge(&ema, &m),
                edge(&ema, &l),
                edge(&atr, &l),
                edge(&u, indicator_node),
                edge(&m, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        "DONCHIAN" => {
            let u = format!("donchian:upper:{}", indicator.period);
            let m = format!("donchian:middle:{}", indicator.period);
            let l = format!("donchian:lower:{}", indicator.period);
            vec![
                edge("high", &u),
                edge("low", &l),
                edge(&u, &m),
                edge(&l, &m),
                edge(&u, indicator_node),
                edge(&m, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        "STOCH_RSI" => {
            let r = format!("rsi:close:{}", indicator.period);
            let s = format!(
                "stoch:rsi:{}:{}:{}:{}",
                indicator.period, indicator.stoch_period, indicator.smooth, indicator.signal
            );
            vec![edge("close", &r), edge(&r, &s), edge(&s, indicator_node)]
        }
        "ADX" => {
            let a = format!("adx:ohlc:{}", indicator.period);
            vec![
                edge("high", &a),
                edge("low", &a),
                edge("close", &a),
                edge(&a, indicator_node),
            ]
        }
        "VWAP" => vec![
            edge("high", "vwap:hlcv"),
            edge("low", "vwap:hlcv"),
            edge("close", "vwap:hlcv"),
            edge("volume", "vwap:hlcv"),
            edge("vwap:hlcv", indicator_node),
        ],
        "PPO" => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let f = format!("ema:close:{}", p.fast);
            let s = format!("ema:close:{}", p.slow);
            let ppo = format!("ppo:{}:{}:{}", p.fast, p.slow, p.signal);
            vec![
                edge("close", &f),
                edge("close", &s),
                edge(&f, &ppo),
                edge(&s, &ppo),
                edge(&ppo, indicator_node),
            ]
        }
        "STOCHASTIC" => {
            let s = format!("stoch:hlc:{}:{}", indicator.period, indicator.smooth);
            vec![
                edge("high", &s),
                edge("low", &s),
                edge("close", &s),
                edge(&s, indicator_node),
            ]
        }
        "MEDIAN_PRICE" => vec![
            edge("high", "median_price:hl"),
            edge("low", "median_price:hl"),
            edge("median_price:hl", indicator_node),
        ],
        "HIGHEST_HIGH" => {
            let h = format!("highest_high:h:{}", indicator.period);
            vec![edge("high", &h), edge(&h, indicator_node)]
        }
        "LOWEST_LOW" => {
            let l = format!("lowest_low:l:{}", indicator.period);
            vec![edge("low", &l), edge(&l, indicator_node)]
        }
        "ALLIGATOR" => vec![
            edge("high", "alligator:jaw"),
            edge("low", "alligator:jaw"),
            edge("high", "alligator:teeth"),
            edge("low", "alligator:teeth"),
            edge("high", "alligator:lips"),
            edge("low", "alligator:lips"),
            edge("alligator:jaw", indicator_node),
            edge("alligator:teeth", indicator_node),
            edge("alligator:lips", indicator_node),
        ],
        "ATR_BANDS" => {
            let ema = format!("ema:close:{}", indicator.period);
            let atr = format!("atr:ohlc:{}", indicator.period);
            let u = format!("atr_bands:upper:{}:{}", indicator.period, indicator.multiplier);
            let l = format!("atr_bands:lower:{}:{}", indicator.period, indicator.multiplier);
            vec![
                edge("close", &ema),
                edge("high", &atr),
                edge("low", &atr),
                edge("close", &atr),
                edge(&ema, &u),
                edge(&atr, &u),
                edge(&ema, &l),
                edge(&atr, &l),
                edge(&u, indicator_node),
                edge(&ema, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        "HIGH_LOW_BANDS" => {
            let u = format!("hlbands:upper:{}", indicator.period);
            let m = format!("hlbands:middle:{}", indicator.period);
            let l = format!("hlbands:lower:{}", indicator.period);
            vec![
                edge("high", &u),
                edge("low", &l),
                edge(&u, &m),
                edge(&l, &m),
                edge(&u, indicator_node),
                edge(&m, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        "FRACTAL_CHAOS_BANDS" => vec![
            edge("high", "fcb:upper"),
            edge("low", "fcb:upper"),
            edge("high", "fcb:lower"),
            edge("low", "fcb:lower"),
            edge("fcb:upper", indicator_node),
            edge("fcb:lower", indicator_node),
        ],
        "GMMA" => {
            let mut edges = Vec::new();
            for &p in &[3, 5, 8, 10, 12, 15, 30, 35, 40, 45, 50, 60] {
                let e = format!("ema:close:{p}");
                edges.push(edge("close", &e));
                edges.push(edge(&e, indicator_node));
            }
            edges
        }
        "LINEAR_REG_FORECAST" => {
            let l = format!("linreg_forecast:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        "LINEAR_REG_INTERCEPT" => {
            let l = format!("linreg_intercept:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        "ANCHORED_VWAP" => {
            let a = format!("anchored_vwap:{}", indicator.anchor);
            vec![
                edge("high", &a),
                edge("low", &a),
                edge("close", &a),
                edge("volume", &a),
                edge(&a, indicator_node),
            ]
        }
        "TYPICAL_PRICE" => vec![
            edge("high", "typical_price:hlc"),
            edge("low", "typical_price:hlc"),
            edge("close", "typical_price:hlc"),
            edge("typical_price:hlc", indicator_node),
        ],
        "WEIGHTED_CLOSE" => vec![
            edge("high", "weighted_close:hlc"),
            edge("low", "weighted_close:hlc"),
            edge("close", "weighted_close:hlc"),
            edge("weighted_close:hlc", indicator_node),
        ],
        "MA_CROSS" => {
            let p = indicator.macd.unwrap_or(MacdParams { fast: indicator.period, slow: indicator.stoch_period, signal: 9 });
            let f = format!("sma:close:{}", p.fast);
            let s = format!("sma:close:{}", p.slow);
            vec![
                edge("close", &f),
                edge("close", &s),
                edge(&f, indicator_node),
                edge(&s, indicator_node),
            ]
        }
        "RAINBOW_MA" => {
            let mut edges = vec![edge("close", &format!("rainbow:r1:{}", indicator.period))];
            for i in 1..10 {
                edges.push(edge(
                    &format!("rainbow:r{}:{}", i, indicator.period),
                    &format!("rainbow:r{}:{}", i + 1, indicator.period),
                ));
            }
            for i in 1..=10 {
                edges.push(edge(&format!("rainbow:r{}:{}", i, indicator.period), indicator_node));
            }
            edges
        }
        "PRIME_NUMBER_BANDS" => vec![
            edge("high", "pnb:upper"),
            edge("low", "pnb:lower"),
            edge("pnb:upper", indicator_node),
            edge("pnb:lower", indicator_node),
        ],
        "TIME_SERIES_FORECAST" => {
            let l = format!("linreg_forecast:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        "VALUATION_LINES" => {
            let sma = format!("sma:close:{}", indicator.period);
            let u = format!("valuation:upper:{}:{}", indicator.period, indicator.multiplier);
            let l = format!("valuation:lower:{}:{}", indicator.period, indicator.multiplier);
            vec![
                edge("close", &sma),
                edge(&sma, &u),
                edge(&sma, &l),
                edge(&u, indicator_node),
                edge(&sma, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        "BETA" => {
            let b = format!("beta:close:{}", indicator.period);
            vec![edge("close", &b), edge(&b, indicator_node)]
        }
        "CORRELATION_COEFFICIENT" => {
            let c = format!("correl:close:{}", indicator.period);
            vec![edge("close", &c), edge(&c, indicator_node)]
        }
        "PERFORMANCE_INDEX" => vec![
            edge("close", "perf_index:close"),
            edge("perf_index:close", indicator_node),
        ],
        "PRICE_RELATIVE" => {
            let p = format!("price_relative:close:{}", indicator.period);
            vec![edge("close", &p), edge(&p, indicator_node)]
        }
        "AWESOME_OSCILLATOR" => vec![
            edge("high", "ao:hl"),
            edge("low", "ao:hl"),
            edge("ao:hl", indicator_node),
        ],
        "BOLLINGER_PCT_B" => {
            let b = format!("bb_pctb:{}:{}", indicator.period, indicator.multiplier);
            vec![edge("close", &b), edge(&b, indicator_node)]
        }
        "CENTER_OF_GRAVITY" => {
            let c = format!("cog:close:{}", indicator.period);
            vec![edge("close", &c), edge(&c, indicator_node)]
        }
        "CHANDE_FORECAST" => {
            let c = format!("cfo:close:{}", indicator.period);
            vec![edge("close", &c), edge(&c, indicator_node)]
        }
        "CHANDE_MOMENTUM" => {
            let c = format!("cmo:close:{}", indicator.period);
            vec![edge("close", &c), edge(&c, indicator_node)]
        }
        "COPPOCK_CURVE" => vec![
            edge("close", "coppock:close"),
            edge("coppock:close", indicator_node),
        ],
        "DISPARITY_INDEX" => {
            let e = format!("ema:close:{}", indicator.period);
            let d = format!("disparity:close:{}", indicator.period);
            vec![edge("close", &e), edge(&e, &d), edge(&d, indicator_node)]
        }
        "EASE_OF_MOVEMENT" => {
            let e = format!("emv:hlv:{}", indicator.period);
            vec![edge("high", &e), edge("low", &e), edge("volume", &e), edge(&e, indicator_node)]
        }
        "EHLER_FISHER" => {
            let f = format!("fisher:hl:{}", indicator.period);
            vec![edge("high", &f), edge("low", &f), edge(&f, indicator_node)]
        }
        "ELDER_RAY" => {
            let e = format!("ema:close:{}", indicator.period);
            let r = format!("elder_ray:hl:{}", indicator.period);
            vec![edge("close", &e), edge("high", &r), edge("low", &r), edge(&e, &r), edge(&r, indicator_node)]
        }
        "FRACTAL_CHAOS_OSCILLATOR" => vec![
            edge("high", "fco:hl"),
            edge("low", "fco:hl"),
            edge("fco:hl", indicator_node),
        ],
        "GATOR_OSCILLATOR" => vec![
            edge("high", "gator:upper"), edge("low", "gator:upper"),
            edge("high", "gator:lower"), edge("low", "gator:lower"),
            edge("gator:upper", indicator_node), edge("gator:lower", indicator_node),
        ],
        "INTRADAY_MOMENTUM" => {
            let i = format!("imi:oc:{}", indicator.period);
            vec![edge("close", &i), edge(&i, indicator_node)]
        }
        "LINEAR_REG_SLOPE" => {
            let l = format!("linreg_slope:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        "MA_DEVIATION" => {
            let s = format!("sma:close:{}", indicator.period);
            let d = format!("ma_dev:close:{}", indicator.period);
            vec![edge("close", &s), edge(&s, &d), edge(&d, indicator_node)]
        }
        "PRETTY_GOOD_OSCILLATOR" => {
            let s = format!("sma:close:{}", indicator.period);
            let a = format!("atr:ohlc:{}", indicator.period);
            let p = format!("pgo:close:{}", indicator.period);
            vec![edge("close", &s), edge("high", &a), edge("low", &a), edge("close", &a),
                 edge(&s, &p), edge(&a, &p), edge(&p, indicator_node)]
        }
        "PRICE_MOMENTUM_OSCILLATOR" => {
            let p = format!("pmo:close:{}:{}", indicator.period, indicator.smooth);
            vec![edge("close", &p), edge(&p, indicator_node)]
        }
        "PRICE_OSCILLATOR" => {
            let params = indicator.macd.unwrap_or(MacdParams { fast: 12, slow: 26, signal: 9 });
            let p = format!("price_osc:close:{}:{}", params.fast, params.slow);
            vec![
                edge("close", &format!("ema:close:{}", params.fast)),
                edge("close", &format!("ema:close:{}", params.slow)),
                edge(&p, indicator_node),
            ]
        }
        "RAINBOW_OSCILLATOR" => {
            let r = format!("rainbow_osc:close:{}", indicator.period);
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        "RAVI" => {
            let r = format!("ravi:close:{}:{}", indicator.period, indicator.stoch_period);
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        "RELATIVE_VIGOR" => {
            let r = format!("rvi:ohlc:{}", indicator.period);
            vec![edge("high", &r), edge("low", &r), edge("close", &r), edge(&r, indicator_node)]
        }
        "SCHAFF_TREND_CYCLE" => {
            let p = indicator.macd.unwrap_or(MacdParams { fast: 12, slow: 26, signal: 9 });
            let s = format!("stc:{}:{}:{}", p.fast, p.slow, indicator.stoch_period);
            vec![edge("close", &s), edge(&s, indicator_node)]
        }
        "STOCHASTIC_MOMENTUM" => {
            let s = format!("smi:hlc:{}:{}", indicator.period, indicator.smooth);
            vec![edge("high", &s), edge("low", &s), edge("close", &s), edge(&s, indicator_node)]
        }
        "SWING_INDEX" => vec![
            edge("high", "swing_index:ohlc"), edge("low", "swing_index:ohlc"),
            edge("close", "swing_index:ohlc"), edge("swing_index:ohlc", indicator_node),
        ],
        "TREND_INTENSITY" => {
            let s = format!("sma:close:{}", indicator.period);
            let t = format!("tii:close:{}", indicator.period);
            vec![edge("close", &s), edge(&s, &t), edge(&t, indicator_node)]
        }
        "VOLUME_OSCILLATOR" => {
            let p = indicator.macd.unwrap_or(MacdParams { fast: 5, slow: 10, signal: 9 });
            let v = format!("vol_osc:volume:{}:{}", p.fast, p.slow);
            vec![edge("volume", &v), edge(&v, indicator_node)]
        }
        "KLINGER_VOLUME" => vec![
            edge("high", "klinger:hlcv"), edge("low", "klinger:hlcv"),
            edge("close", "klinger:hlcv"), edge("volume", "klinger:hlcv"),
            edge("klinger:hlcv", indicator_node),
        ],
        "MARKET_FACILITATION" => vec![
            edge("high", "mfi_bw:hlv"), edge("low", "mfi_bw:hlv"),
            edge("volume", "mfi_bw:hlv"), edge("mfi_bw:hlv", indicator_node),
        ],
        "NEGATIVE_VOLUME_INDEX" => vec![
            edge("close", "nvi:cv"), edge("volume", "nvi:cv"), edge("nvi:cv", indicator_node),
        ],
        "POSITIVE_VOLUME_INDEX" => vec![
            edge("close", "pvi:cv"), edge("volume", "pvi:cv"), edge("pvi:cv", indicator_node),
        ],
        "PRICE_VOLUME_TREND" => vec![
            edge("close", "pvt:cv"), edge("volume", "pvt:cv"), edge("pvt:cv", indicator_node),
        ],
        "TRADE_VOLUME_INDEX" => vec![
            edge("close", "tvi:cv"), edge("volume", "tvi:cv"), edge("tvi:cv", indicator_node),
        ],
        "TWIGGS_MONEY_FLOW" => {
            let t = format!("tmf:hlcv:{}", indicator.period);
            vec![edge("high", &t), edge("low", &t), edge("close", &t), edge("volume", &t), edge(&t, indicator_node)]
        }
        "PROJECTED_AGGREGATE_VOLUME" => {
            let p = format!("pav:v:{}", indicator.period);
            vec![edge("volume", &p), edge(&p, indicator_node)]
        }
        "PROJECTED_VOLUME_AT_TIME" => {
            let p = format!("pvat:v:{}", indicator.period);
            vec![edge("volume", &p), edge(&p, indicator_node)]
        }
        "HISTORICAL_VOLATILITY" => {
            let h = format!("hv:close:{}", indicator.period);
            vec![edge("close", &h), edge(&h, indicator_node)]
        }
        "LINEAR_REG_R2" => {
            let r = format!("linreg_r2:close:{}", indicator.period);
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        "PRIME_NUMBER_OSCILLATOR" => vec![
            edge("close", "pno:close"), edge("pno:close", indicator_node),
        ],
        "RANDOM_WALK_INDEX" => {
            let r = format!("rwi:hlc:{}", indicator.period);
            vec![edge("high", &r), edge("low", &r), edge("close", &r), edge(&r, indicator_node)]
        }
        "DARVAS_BOX" => vec![
            edge("high", "darvas:top"), edge("low", "darvas:bottom"),
            edge("darvas:top", indicator_node), edge("darvas:bottom", indicator_node),
        ],
        "VOLUME_PROFILE" => {
            let v = format!("vol_profile:hlv:{}", indicator.period);
            vec![edge("high", &v), edge("low", &v), edge("volume", &v), edge(&v, indicator_node)]
        }
        "CHOPPINESS_INDEX" => {
            let c = format!("chop:hlc:{}", indicator.period);
            vec![edge("high", &c), edge("low", &c), edge("close", &c), edge(&c, indicator_node)]
        }
        "ELDER_IMPULSE" => {
            let e = format!("impulse:close:{}", indicator.period);
            vec![edge("close", &e), edge(&e, indicator_node)]
        }
        "GONOGO_TREND" => {
            let g = format!("gonogo:close:{}", indicator.period);
            vec![edge("close", &g), edge(&g, indicator_node)]
        }
        "PSYCHOLOGICAL_LINE" => {
            let p = format!("psy:close:{}", indicator.period);
            vec![edge("close", &p), edge(&p, indicator_node)]
        }
        "QSTICK" => {
            let q = format!("qstick:oc:{}", indicator.period);
            vec![edge("close", &q), edge(&q, indicator_node)]
        }
        "SHINOHARA_INTENSITY" => {
            let s = format!("shinohara:hlc:{}", indicator.period);
            vec![edge("high", &s), edge("low", &s), edge("close", &s), edge(&s, indicator_node)]
        }
        "ULCER_INDEX" => {
            let u = format!("ulcer:close:{}", indicator.period);
            vec![edge("close", &u), edge(&u, indicator_node)]
        }
        "VERTICAL_HORIZONTAL_FILTER" => {
            let v = format!("vhf:close:{}", indicator.period);
            vec![edge("close", &v), edge(&v, indicator_node)]
        }
        "VORTEX_INDICATOR" => {
            let v = format!("vortex:hlc:{}", indicator.period);
            vec![edge("high", &v), edge("low", &v), edge("close", &v), edge(&v, indicator_node)]
        }
        "ZIGZAG" => {
            let z = format!("zigzag:hl:{}", indicator.multiplier);
            vec![edge("high", &z), edge("low", &z), edge(&z, indicator_node)]
        }
        "BOLLINGER_BANDWIDTH" => {
            let b = format!("bb_bw:{}:{}", indicator.period, indicator.multiplier);
            vec![edge("close", &b), edge(&b, indicator_node)]
        }
        "DONCHIAN_WIDTH" => {
            let d = format!("donchian_width:{}", indicator.period);
            vec![edge("high", &d), edge("low", &d), edge(&d, indicator_node)]
        }
        "GOPALAKRISHNAN_RANGE" => {
            let g = format!("gapo:hl:{}", indicator.period);
            vec![edge("high", &g), edge("low", &g), edge(&g, indicator_node)]
        }
        "HIGH_MINUS_LOW" => vec![edge("high", "hml:hl"), edge("low", "hml:hl"), edge("hml:hl", indicator_node)],
        "MASS_INDEX" => {
            let m = format!("mass:hl:{}", indicator.period);
            vec![edge("high", &m), edge("low", &m), edge(&m, indicator_node)]
        }
        "RELATIVE_VOLATILITY" => {
            let r = format!("rvi_vol:close:{}", indicator.period);
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        "TRUE_RANGE" => vec![
            edge("high", "true_range:hlc"), edge("low", "true_range:hlc"),
            edge("close", "true_range:hlc"), edge("true_range:hlc", indicator_node),
        ],
        "VOLUME_CHART" => vec![edge("volume", "vol_chart:v"), edge("vol_chart:v", indicator_node)],
        "VOLUME_ROC" => {
            let v = format!("vol_roc:v:{}", indicator.period);
            vec![edge("volume", &v), edge(&v, indicator_node)]
        }
        "VOLUME_UNDERLAY" => vec![
            edge("close", "vol_underlay:cv"), edge("volume", "vol_underlay:cv"),
            edge("vol_underlay:cv", indicator_node),
        ],
        _ => indicator_nodes(indicator)
            .into_iter()
            .map(|node| edge(&node, indicator_node))
            .collect(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_indicator(
    kind: &str,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    macd: Option<MacdParams>,
    multiplier: f64,
    psar_step: f64,
    psar_max_step: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    if kind == "MACD" || kind == "PPO" || kind == "CHAIKIN_OSCILLATOR" || kind == "MA_CROSS" || kind == "PRICE_OSCILLATOR" || kind == "VOLUME_OSCILLATOR" || kind == "SCHAFF_TREND_CYCLE" {
        let macd = macd.expect("MACD params are built before validation");
        if macd.fast == 0 || macd.slow <= macd.fast || macd.signal == 0 {
            return Err(wasm_bindgen::JsValue::from_str(
                "fast/slow params must satisfy fast > 0, slow > fast, signal > 0",
            ));
        }
    } else if kind == "ICHIMOKU"
        && (tenkan_period == 0
            || kijun_period == 0
            || senkou_b_period == 0
            || kijun_period < tenkan_period
            || senkou_b_period < kijun_period)
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "ICHIMOKU params must satisfy tenkan > 0, kijun >= tenkan, senkou_b >= kijun",
        ));
    } else if kind == "PARABOLIC_SAR"
        && (!psar_step.is_finite()
            || !psar_max_step.is_finite()
            || psar_step <= 0.0
            || psar_max_step <= 0.0
            || psar_max_step < psar_step)
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "PARABOLIC_SAR params must satisfy step > 0 and max_step >= step",
        ));
    } else if needs_period(kind) && period == 0
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "period must be greater than zero",
        ));
    } else if (kind == "STOCH_RSI" || kind == "TSI") && stoch_period == 0 {
        return Err(wasm_bindgen::JsValue::from_str(
            "stoch_period must be greater than zero",
        ));
    } else if (kind == "STOCHASTIC" || kind == "STOCH_RSI") && smooth == 0 {
        return Err(wasm_bindgen::JsValue::from_str(
            "smooth must be greater than zero",
        ));
    } else if kind == "STOCH_RSI" && signal == 0 {
        return Err(wasm_bindgen::JsValue::from_str(
            "signal must be greater than zero",
        ));
    } else if kind == "ULTIMATE_OSCILLATOR" && (stoch_period < period || smooth < stoch_period) {
        return Err(wasm_bindgen::JsValue::from_str(
            "ULTIMATE_OSCILLATOR params must satisfy short <= medium <= long",
        ));
    } else if (kind == "BB"
        || kind == "SUPERTREND"
        || kind == "KELTNER"
        || kind == "ENVELOPE"
        || kind == "STARC"
        || kind == "ATR_BANDS")
        && (!multiplier.is_finite() || multiplier <= 0.0)
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "multiplier must be greater than zero",
        ));
    }
    Ok(())
}

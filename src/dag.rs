use crate::indicators::klinger_volume::KLINGER_DEFAULTS;
use crate::indicators::trima::triangular_windows;
use crate::types::{DagEdge, Indicator, IndicatorKind, MacdParams};

pub(crate) fn is_valid_kind(kind: &str) -> bool {
    kind.parse::<IndicatorKind>().is_ok()
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
            | "ema_state"
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
    match (indicator.kind, indicator.macd) {
        (IndicatorKind::MACD, Some(macd)) | (IndicatorKind::PPO, Some(macd)) => format!(
            "{}({},{},{})#{}",
            indicator.kind, macd.fast, macd.slow, macd.signal, indicator.id
        ),
        (IndicatorKind::CHAIKIN_OSCILLATOR, Some(macd)) => format!(
            "CHAIKIN_OSCILLATOR({},{})#{}",
            macd.fast, macd.slow, indicator.id
        ),
        _ => format!("{}#{}", indicator.kind, indicator.id),
    }
}

/// Cache key of the Klinger oscillator line (the default periods keep the original key).
fn klinger_node(p: MacdParams) -> String {
    if p.fast == KLINGER_DEFAULTS.fast && p.slow == KLINGER_DEFAULTS.slow {
        "klinger_sv:hlcv".to_string()
    } else {
        format!("klinger_sv:hlcv:{}:{}", p.fast, p.slow)
    }
}

pub(crate) fn edge(from: &str, to: &str) -> DagEdge {
    DagEdge {
        from: from.to_string(),
        to: to.to_string(),
    }
}

pub(crate) fn indicator_nodes(indicator: &Indicator) -> Vec<String> {
    match indicator.kind {
        IndicatorKind::SMA => vec![format!("sma:close:{}", indicator.period)],
        IndicatorKind::EMA => vec![format!("ema_sma:close:{}", indicator.period)],
        IndicatorKind::WMA => vec![format!("wma:close:{}", indicator.period)],
        IndicatorKind::HMA => vec![
            format!("wma:close:{}", indicator.period / 2),
            format!("wma:close:{}", indicator.period),
            format!("hma:close:{}", indicator.period),
        ],
        IndicatorKind::LINEAR_REGRESSION => vec![format!("linreg:close:{}", indicator.period)],
        IndicatorKind::DEMA => vec![
            format!("ema_sma:close:{}", indicator.period),
            format!("dema_sma:ema2:{}", indicator.period),
            format!("dema_sma:value:{}", indicator.period),
        ],
        IndicatorKind::TEMA => vec![
            format!("ema_sma:close:{}", indicator.period),
            format!("tema_sma:ema2:{}", indicator.period),
            format!("tema_sma:ema3:{}", indicator.period),
            format!("tema_sma:value:{}", indicator.period),
        ],
        IndicatorKind::TRIMA => vec![
            format!("sma:close:{}", triangular_windows(indicator.period).0),
            format!("trima:value:{}", indicator.period),
        ],
        IndicatorKind::STDDEV => vec![format!("stddev:close:{}", indicator.period)],
        IndicatorKind::ENVELOPE => vec![
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
        IndicatorKind::TRIX => vec![
            format!("ema_sma:close:{}", indicator.period),
            format!("trix_sma:ema2:{}", indicator.period),
            format!("trix_sma:ema3:{}", indicator.period),
            format!("trix_sma:value:{}", indicator.period),
        ],
        IndicatorKind::TSI => vec![format!(
            "tsi:{}:{}",
            indicator.period, indicator.stoch_period
        )],
        IndicatorKind::DPO => vec![
            format!("sma:close:{}", indicator.period),
            format!("dpo:close:{}", indicator.period),
        ],
        IndicatorKind::MOMENTUM => vec![format!("momentum:close:{}", indicator.period)],
        IndicatorKind::RSI => vec![format!("rsi:close:{}", indicator.period)],
        IndicatorKind::ROC => vec![format!("roc:close:{}", indicator.period)],
        IndicatorKind::CCI => vec![format!("cci:hlc:{}", indicator.period)],
        IndicatorKind::CMF => vec![format!("cmf:hlcv:{}", indicator.period)],
        IndicatorKind::MFI => vec![format!("mfi:hlcv:{}", indicator.period)],
        IndicatorKind::WILLIAMS_R => vec![format!("willr:hlc:{}", indicator.period)],
        IndicatorKind::VWMA => vec![format!("vwma:close:volume:{}", indicator.period)],
        IndicatorKind::WILLIAMS_AD => vec!["wad:ohlc".to_string()],
        IndicatorKind::CHAIKIN_VOLATILITY => vec![
            format!("cvol_sma:ma:{}", indicator.period),
            format!("cvol_sma:value:{}:{}", indicator.period, indicator.smooth),
        ],
        IndicatorKind::PRICE_CHANNEL => vec![
            format!("price_channel:upper:{}", indicator.period),
            format!("price_channel:middle:{}", indicator.period),
            format!("price_channel:lower:{}", indicator.period),
        ],
        IndicatorKind::STARC => {
            let key = |part: &str| {
                format!(
                    "starc:{part}:{}:{}:{}",
                    indicator.period, indicator.smooth, indicator.multiplier
                )
            };
            vec![
                format!("sma:close:{}", indicator.smooth),
                format!("atr:ohlc:{}", indicator.period),
                key("upper"),
                key("middle"),
                key("lower"),
            ]
        }
        IndicatorKind::PARABOLIC_SAR => vec![format!(
            "psar:ohlc:{}:{}",
            indicator.psar_step, indicator.psar_max_step
        )],
        IndicatorKind::ICHIMOKU => vec![
            format!("ichimoku:tenkan:{}", indicator.tenkan_period),
            format!("ichimoku:kijun:{}", indicator.kijun_period),
            format!(
                "ichimoku:senkou_a:{}:{}",
                indicator.tenkan_period, indicator.kijun_period
            ),
            format!("ichimoku:senkou_b:{}", indicator.senkou_b_period),
            "ichimoku:chikou".to_string(),
        ],
        IndicatorKind::PIVOT_POINTS => vec![
            "pivot:pp".to_string(),
            "pivot:r1".to_string(),
            "pivot:s1".to_string(),
            "pivot:r2".to_string(),
            "pivot:s2".to_string(),
            "pivot:r3".to_string(),
            "pivot:s3".to_string(),
        ],
        IndicatorKind::AROON => vec![format!("aroon_lookback:hl:{}", indicator.period)],
        IndicatorKind::ADL => vec!["adl:hlcv".to_string()],
        IndicatorKind::KST => vec![
            "roc:close:10".to_string(),
            "roc:close:15".to_string(),
            "roc:close:20".to_string(),
            "roc:close:30".to_string(),
            "kst:value".to_string(),
            format!("kst:signal:{}", indicator.signal),
        ],
        IndicatorKind::BOP => vec![
            "bop:ohlc".to_string(),
            format!("bop:ohlc:sma:{}", indicator.period),
        ],
        IndicatorKind::ULTIMATE_OSCILLATOR => vec![format!(
            "uo:{}:{}:{}",
            indicator.period, indicator.stoch_period, indicator.smooth
        )],
        IndicatorKind::CHAIKIN_OSCILLATOR => {
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
        IndicatorKind::FORCE_INDEX => vec![format!("force_sma:close:volume:{}", indicator.period)],
        IndicatorKind::SUPERTREND => vec![
            format!("atr:ohlc:{}", indicator.period),
            format!("supertrend:{}:{}", indicator.period, indicator.multiplier),
        ],
        IndicatorKind::KELTNER => vec![
            format!("ema_sma:close:{}", indicator.period),
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
        IndicatorKind::DONCHIAN => vec![
            format!("donchian:upper:{}", indicator.period),
            format!("donchian:middle:{}", indicator.period),
            format!("donchian:lower:{}", indicator.period),
        ],
        IndicatorKind::STOCH_RSI => vec![
            format!("rsi:close:{}", indicator.period),
            format!(
                "stoch:rsi:{}:{}:{}:{}",
                indicator.period, indicator.stoch_period, indicator.smooth, indicator.signal
            ),
        ],
        IndicatorKind::ATR => vec![format!("atr:ohlc:{}", indicator.period)],
        IndicatorKind::ADX => vec![format!("adx:ohlc:{}", indicator.period)],
        IndicatorKind::VWAP => vec!["vwap:hlcv".to_string()],
        IndicatorKind::STOCHASTIC => vec![format!(
            "stoch:hlc:{}:{}:{}",
            indicator.period, indicator.smooth, indicator.signal
        )],
        IndicatorKind::BB => vec![
            format!("sma:close:{}", indicator.period),
            format!("bb:upper:{}:{}", indicator.period, indicator.multiplier),
            format!("bb:middle:{}:{}", indicator.period, indicator.multiplier),
            format!("bb:lower:{}:{}", indicator.period, indicator.multiplier),
        ],
        IndicatorKind::OBV => vec!["obv:close:volume".to_string()],
        IndicatorKind::MACD => {
            let m = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            vec![
                format!("ema_sma:close:{}", m.fast),
                format!("ema:close:{}", m.slow),
            ]
        }
        IndicatorKind::PPO => {
            let m = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            vec![
                format!("ema_sma:close:{}", m.fast),
                format!("ema_sma:close:{}", m.slow),
                format!("ppo:{}:{}:{}", m.fast, m.slow, m.signal),
            ]
        }
        IndicatorKind::MEDIAN_PRICE => vec![format!("median_price:hl:{}", indicator.period)],
        IndicatorKind::HIGHEST_HIGH => vec![format!("highest_high:h:{}", indicator.period)],
        IndicatorKind::LOWEST_LOW => vec![format!("lowest_low:l:{}", indicator.period)],
        IndicatorKind::ALLIGATOR => vec![
            "alligator_seeded:jaw".to_string(),
            "alligator_seeded:teeth".to_string(),
            "alligator_seeded:lips".to_string(),
        ],
        IndicatorKind::ATR_BANDS => vec![
            format!("atr:ohlc:{}", indicator.period),
            format!(
                "atr_bands:upper:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "atr_bands:lower:{}:{}",
                indicator.period, indicator.multiplier
            ),
        ],
        IndicatorKind::HIGH_LOW_BANDS => vec![
            format!("hlbands:middle:{}", indicator.period),
            format!(
                "hlbands:upper:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "hlbands:lower:{}:{}",
                indicator.period, indicator.multiplier
            ),
        ],
        IndicatorKind::FRACTAL_CHAOS_BANDS => {
            vec!["fcb_ties:upper".to_string(), "fcb_ties:lower".to_string()]
        }
        IndicatorKind::GMMA => {
            let mut nodes = Vec::with_capacity(12);
            for &p in &[3, 5, 8, 10, 12, 15, 30, 35, 40, 45, 50, 60] {
                nodes.push(format!("ema_sma:close:{p}"));
            }
            nodes
        }
        IndicatorKind::LINEAR_REG_FORECAST => {
            vec![format!(
                "linreg_forecast_current:close:{}",
                indicator.period
            )]
        }
        IndicatorKind::LINEAR_REG_INTERCEPT => {
            vec![format!("linreg_intercept:close:{}", indicator.period)]
        }
        IndicatorKind::ANCHORED_VWAP => vec![format!("anchored_vwap:{}", indicator.anchor)],
        IndicatorKind::TYPICAL_PRICE => vec![format!("typical_price:hlc:{}", indicator.period)],
        IndicatorKind::WEIGHTED_CLOSE => vec![format!("weighted_close:hlc:{}", indicator.period)],
        IndicatorKind::MA_CROSS => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: indicator.period,
                slow: indicator.stoch_period,
                signal: 9,
            });
            vec![
                format!("sma:close:{}", p.fast),
                format!("sma:close:{}", p.slow),
            ]
        }
        IndicatorKind::RAINBOW_MA => (1..=10)
            .map(|i| format!("rainbow:r{}:{}", i, indicator.period))
            .collect(),
        IndicatorKind::PRIME_NUMBER_BANDS => vec![
            "pnb_strict:upper".to_string(),
            "pnb_strict:lower".to_string(),
        ],
        IndicatorKind::TIME_SERIES_FORECAST => {
            vec![format!("tsf:close:{}", indicator.period)]
        }
        IndicatorKind::VALUATION_LINES => vec![
            format!("sma:close:{}", indicator.period),
            format!(
                "valuation:upper:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "valuation:lower:{}:{}",
                indicator.period, indicator.multiplier
            ),
        ],
        IndicatorKind::BETA => vec![format!("beta:close:{}", indicator.period)],
        IndicatorKind::CORRELATION_COEFFICIENT => {
            vec![format!("correl:close:{}", indicator.period)]
        }
        IndicatorKind::PERFORMANCE_INDEX => vec!["perf_index:close".to_string()],
        IndicatorKind::PRICE_RELATIVE => vec![format!("price_relative:close:{}", indicator.period)],
        IndicatorKind::AWESOME_OSCILLATOR => vec!["ao:hl".to_string()],
        IndicatorKind::BOLLINGER_PCT_B => vec![format!(
            "bb_pctb100:{}:{}",
            indicator.period, indicator.multiplier
        )],
        IndicatorKind::CENTER_OF_GRAVITY => vec![format!("cog_ehlers:close:{}", indicator.period)],
        IndicatorKind::CHANDE_FORECAST => vec![format!("cfo:close:{}", indicator.period)],
        IndicatorKind::CHANDE_MOMENTUM => vec![format!("cmo:close:{}", indicator.period)],
        IndicatorKind::COPPOCK_CURVE => vec!["coppock:close".to_string()],
        IndicatorKind::DISPARITY_INDEX => vec![
            format!("sma:close:{}", indicator.period),
            format!("disparity_sma:close:{}", indicator.period),
        ],
        IndicatorKind::EASE_OF_MOVEMENT => vec![format!("emv_scaled:hlv:{}", indicator.period)],
        IndicatorKind::EHLER_FISHER => vec![format!("fisher_mid:hl:{}", indicator.period)],
        IndicatorKind::ELDER_RAY => vec![
            format!("ema_sma:close:{}", indicator.period),
            format!("elder_ray:hl:{}", indicator.period),
        ],
        IndicatorKind::FRACTAL_CHAOS_OSCILLATOR => vec!["fco_ties:hl".to_string()],
        IndicatorKind::GATOR_OSCILLATOR => vec![
            "alligator_seeded:jaw".to_string(),
            "alligator_seeded:teeth".to_string(),
            "alligator_seeded:lips".to_string(),
            "gator_drawn:upper".to_string(),
            "gator_drawn:lower".to_string(),
        ],
        IndicatorKind::INTRADAY_MOMENTUM => vec![format!("imi:oc:{}", indicator.period)],
        IndicatorKind::LINEAR_REG_SLOPE => vec![format!("linreg_slope:close:{}", indicator.period)],
        IndicatorKind::MA_DEVIATION => vec![
            format!("sma:close:{}", indicator.period),
            format!("ma_dev:close:{}", indicator.period),
        ],
        IndicatorKind::PRETTY_GOOD_OSCILLATOR => vec![
            format!("sma:close:{}", indicator.period),
            "true_range:hlc".to_string(),
            format!("pgo_ema_tr:close:{}", indicator.period),
        ],
        IndicatorKind::PRICE_MOMENTUM_OSCILLATOR => vec![format!(
            "pmo_dp:close:{}:{}",
            indicator.period, indicator.smooth
        )],
        IndicatorKind::PRICE_OSCILLATOR => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            vec![format!("price_osc_pts:close:{}:{}", p.fast, p.slow)]
        }
        IndicatorKind::RAINBOW_OSCILLATOR => {
            vec![format!(
                "rainbow_osc_range:close:{}:{}",
                indicator.period, indicator.smooth
            )]
        }
        IndicatorKind::RAVI => vec![format!(
            "ravi:close:{}:{}",
            indicator.period, indicator.stoch_period
        )],
        IndicatorKind::RELATIVE_VIGOR => vec![format!("rvi:ohlc:{}", indicator.period)],
        IndicatorKind::SCHAFF_TREND_CYCLE => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            vec![format!(
                "stc_seeded:{}:{}:{}",
                p.fast, p.slow, indicator.stoch_period
            )]
        }
        IndicatorKind::STOCHASTIC_MOMENTUM => {
            vec![format!(
                "smi_sma:hlc:{}:{}",
                indicator.period, indicator.smooth
            )]
        }
        IndicatorKind::SWING_INDEX => {
            vec![format!("swing_index_wilder:ohlc:{}", indicator.multiplier)]
        }
        IndicatorKind::TREND_INTENSITY => vec![
            format!("sma:close:{}", indicator.period),
            format!("tii:close:{}", indicator.period),
        ],
        IndicatorKind::VOLUME_OSCILLATOR => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            vec![format!("vol_osc_pts:volume:{}:{}", p.fast, p.slow)]
        }
        IndicatorKind::KLINGER_VOLUME => {
            let p = indicator.macd.unwrap_or(KLINGER_DEFAULTS);
            vec![
                klinger_node(p),
                format!("klinger:signal:{}:{}:{}", p.fast, p.slow, p.signal),
            ]
        }
        IndicatorKind::MARKET_FACILITATION => vec![format!("mfi_bw:hlv:{}", indicator.multiplier)],
        IndicatorKind::NEGATIVE_VOLUME_INDEX => vec![
            "nvi100:cv".to_string(),
            format!("nvi100:cv:sma:{}", indicator.period),
        ],
        IndicatorKind::POSITIVE_VOLUME_INDEX => vec![
            "pvi100:cv".to_string(),
            format!("pvi100:cv:sma:{}", indicator.period),
        ],
        IndicatorKind::PRICE_VOLUME_TREND => vec!["pvt:cv".to_string()],
        IndicatorKind::TRADE_VOLUME_INDEX => vec!["tvi:cv".to_string()],
        IndicatorKind::TWIGGS_MONEY_FLOW => vec![format!("tmf:hlcv:{}", indicator.period)],
        IndicatorKind::PROJECTED_AGGREGATE_VOLUME => vec![format!("pav:v:{}", indicator.period)],
        IndicatorKind::PROJECTED_VOLUME_AT_TIME => vec![format!("pvat:v:{}", indicator.period)],
        IndicatorKind::HISTORICAL_VOLATILITY => vec![format!("hv_pop:close:{}", indicator.period)],
        IndicatorKind::LINEAR_REG_R2 => vec![format!("linreg_r2:close:{}", indicator.period)],
        IndicatorKind::PRIME_NUMBER_OSCILLATOR => {
            vec![format!("pno_bands:hlc:{}", indicator.multiplier)]
        }
        IndicatorKind::RANDOM_WALK_INDEX => vec![format!("rwi:hlc:{}", indicator.period)],
        IndicatorKind::DARVAS_BOX => vec!["darvas:top".to_string(), "darvas:bottom".to_string()],
        IndicatorKind::VOLUME_PROFILE => vec![format!("vol_profile:hlv:{}", indicator.period)],
        IndicatorKind::CHOPPINESS_INDEX => vec![format!("chop:hlc:{}", indicator.period)],
        IndicatorKind::ELDER_IMPULSE => vec![format!("impulse:close:{}", indicator.period)],
        IndicatorKind::GONOGO_TREND => vec![format!("gonogo:close:{}", indicator.period)],
        IndicatorKind::PSYCHOLOGICAL_LINE => vec![format!("psy:close:{}", indicator.period)],
        IndicatorKind::QSTICK => vec![format!("qstick:oc:{}", indicator.period)],
        IndicatorKind::SHINOHARA_INTENSITY => vec![format!("shinohara:hlc:{}", indicator.period)],
        IndicatorKind::ULCER_INDEX => vec![format!("ulcer:close:{}", indicator.period)],
        IndicatorKind::VERTICAL_HORIZONTAL_FILTER => {
            vec![format!("vhf:close:{}", indicator.period)]
        }
        IndicatorKind::VORTEX_INDICATOR => vec![format!("vortex:hlc:{}", indicator.period)],
        IndicatorKind::ZIGZAG => vec![format!("zigzag:hl:{}", indicator.multiplier)],
        IndicatorKind::BOLLINGER_BANDWIDTH => vec![format!(
            "bb_bw:{}:{}",
            indicator.period, indicator.multiplier
        )],
        IndicatorKind::DONCHIAN_WIDTH => {
            vec![format!("donchian_width_points:{}", indicator.period)]
        }
        IndicatorKind::GOPALAKRISHNAN_RANGE => vec![format!("gapo:hl:{}", indicator.period)],
        IndicatorKind::HIGH_MINUS_LOW => vec!["hml:hl".to_string()],
        IndicatorKind::MASS_INDEX => vec![format!("mass_sma:hl:{}", indicator.period)],
        IndicatorKind::RELATIVE_VOLATILITY => vec![format!("rvi_vol:close:{}", indicator.period)],
        IndicatorKind::TRUE_RANGE => vec!["true_range:hlc".to_string()],
        IndicatorKind::VOLUME_CHART => vec!["vol_chart:v".to_string()],
        IndicatorKind::VOLUME_ROC => vec![format!("vol_roc:v:{}", indicator.period)],
        IndicatorKind::VOLUME_UNDERLAY => vec!["vol_underlay:cv".to_string()],
    }
}

pub(crate) fn indicator_edges(indicator: &Indicator, indicator_node: &str) -> Vec<DagEdge> {
    match indicator.kind {
        IndicatorKind::BB => {
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
        IndicatorKind::OBV => vec![
            edge("close", "obv:close:volume"),
            edge("volume", "obv:close:volume"),
            edge("obv:close:volume", indicator_node),
        ],
        IndicatorKind::ATR => {
            let a = format!("atr:ohlc:{}", indicator.period);
            vec![
                edge("high", &a),
                edge("low", &a),
                edge("close", &a),
                edge(&a, indicator_node),
            ]
        }
        IndicatorKind::CCI => {
            let c = format!("cci:hlc:{}", indicator.period);
            vec![edge("hlc3", &c), edge(&c, indicator_node)]
        }
        IndicatorKind::VWMA => {
            let v = format!("vwma:close:volume:{}", indicator.period);
            vec![
                edge("close", &v),
                edge("volume", &v),
                edge(&v, indicator_node),
            ]
        }
        IndicatorKind::WILLIAMS_AD => vec![
            edge("high", "wad:ohlc"),
            edge("low", "wad:ohlc"),
            edge("close", "wad:ohlc"),
            edge("wad:ohlc", indicator_node),
        ],
        IndicatorKind::CHAIKIN_VOLATILITY => {
            let ema = format!("cvol_sma:ma:{}", indicator.period);
            let val = format!("cvol_sma:value:{}:{}", indicator.period, indicator.smooth);
            vec![
                edge("high", &ema),
                edge("low", &ema),
                edge(&ema, &val),
                edge(&val, indicator_node),
            ]
        }
        IndicatorKind::PRICE_CHANNEL => {
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
        IndicatorKind::STARC => {
            let sma = format!("sma:close:{}", indicator.smooth);
            let atr = format!("atr:ohlc:{}", indicator.period);
            let key = |part: &str| {
                format!(
                    "starc:{part}:{}:{}:{}",
                    indicator.period, indicator.smooth, indicator.multiplier
                )
            };
            let (u, m, l) = (key("upper"), key("middle"), key("lower"));
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
        IndicatorKind::DEMA => {
            let e1 = format!("ema_sma:close:{}", indicator.period);
            let e2 = format!("dema_sma:ema2:{}", indicator.period);
            let d = format!("dema_sma:value:{}", indicator.period);
            vec![
                edge("close", &e1),
                edge(&e1, &e2),
                edge(&e2, &d),
                edge(&d, indicator_node),
            ]
        }
        IndicatorKind::TEMA => {
            let e1 = format!("ema_sma:close:{}", indicator.period);
            let e2 = format!("tema_sma:ema2:{}", indicator.period);
            let e3 = format!("tema_sma:ema3:{}", indicator.period);
            let t = format!("tema_sma:value:{}", indicator.period);
            vec![
                edge("close", &e1),
                edge(&e1, &e2),
                edge(&e2, &e3),
                edge(&e3, &t),
                edge(&t, indicator_node),
            ]
        }
        IndicatorKind::TRIMA => {
            let s = format!("sma:close:{}", triangular_windows(indicator.period).0);
            let t = format!("trima:value:{}", indicator.period);
            vec![edge("close", &s), edge(&s, &t), edge(&t, indicator_node)]
        }
        IndicatorKind::STDDEV => {
            let s = format!("stddev:close:{}", indicator.period);
            vec![edge("close", &s), edge(&s, indicator_node)]
        }
        IndicatorKind::ENVELOPE => {
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
        IndicatorKind::WMA => {
            let w = format!("wma:close:{}", indicator.period);
            vec![edge("close", &w), edge(&w, indicator_node)]
        }
        IndicatorKind::HMA => {
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
        IndicatorKind::LINEAR_REGRESSION => {
            let l = format!("linreg:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        IndicatorKind::TRIX => {
            let e1 = format!("ema_sma:close:{}", indicator.period);
            let e2 = format!("trix_sma:ema2:{}", indicator.period);
            let e3 = format!("trix_sma:ema3:{}", indicator.period);
            let t = format!("trix_sma:value:{}", indicator.period);
            vec![
                edge("close", &e1),
                edge(&e1, &e2),
                edge(&e2, &e3),
                edge(&e3, &t),
                edge(&t, indicator_node),
            ]
        }
        IndicatorKind::TSI => {
            let t = format!("tsi:{}:{}", indicator.period, indicator.stoch_period);
            vec![edge("close", &t), edge(&t, indicator_node)]
        }
        IndicatorKind::DPO => {
            let s = format!("sma:close:{}", indicator.period);
            let d = format!("dpo:close:{}", indicator.period);
            vec![
                edge("close", &s),
                edge("close", &d),
                edge(&s, &d),
                edge(&d, indicator_node),
            ]
        }
        IndicatorKind::MOMENTUM => {
            let m = format!("momentum:close:{}", indicator.period);
            vec![edge("close", &m), edge(&m, indicator_node)]
        }
        IndicatorKind::ROC => {
            let r = format!("roc:close:{}", indicator.period);
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        IndicatorKind::WILLIAMS_R => {
            let w = format!("willr:hlc:{}", indicator.period);
            vec![
                edge("high", &w),
                edge("low", &w),
                edge("close", &w),
                edge(&w, indicator_node),
            ]
        }
        IndicatorKind::MFI => {
            let m = format!("mfi:hlcv:{}", indicator.period);
            vec![
                edge("high", &m),
                edge("low", &m),
                edge("close", &m),
                edge("volume", &m),
                edge(&m, indicator_node),
            ]
        }
        IndicatorKind::CMF => {
            let c = format!("cmf:hlcv:{}", indicator.period);
            vec![
                edge("high", &c),
                edge("low", &c),
                edge("close", &c),
                edge("volume", &c),
                edge(&c, indicator_node),
            ]
        }
        IndicatorKind::SUPERTREND => {
            let a = format!("atr:ohlc:{}", indicator.period);
            let s = format!("supertrend:{}:{}", indicator.period, indicator.multiplier);
            vec![
                edge("high", &a),
                edge("low", &a),
                edge("close", &a),
                edge("hl2", &s),
                edge("close", &s),
                edge(&a, &s),
                edge(&s, indicator_node),
            ]
        }
        IndicatorKind::PARABOLIC_SAR => {
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
        IndicatorKind::ICHIMOKU => {
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
        IndicatorKind::PIVOT_POINTS => vec![
            edge("high", "pivot:pp"),
            edge("low", "pivot:pp"),
            edge("close", "pivot:pp"),
            edge("pivot:pp", "pivot:r1"),
            edge("pivot:pp", "pivot:s1"),
            edge("pivot:pp", "pivot:r2"),
            edge("pivot:pp", "pivot:s2"),
            edge("pivot:pp", "pivot:r3"),
            edge("pivot:pp", "pivot:s3"),
            edge("pivot:pp", indicator_node),
            edge("pivot:r1", indicator_node),
            edge("pivot:s1", indicator_node),
            edge("pivot:r2", indicator_node),
            edge("pivot:s2", indicator_node),
            edge("pivot:r3", indicator_node),
            edge("pivot:s3", indicator_node),
        ],
        IndicatorKind::AROON => {
            let a = format!("aroon_lookback:hl:{}", indicator.period);
            vec![edge("high", &a), edge("low", &a), edge(&a, indicator_node)]
        }
        IndicatorKind::ADL => vec![
            edge("high", "adl:hlcv"),
            edge("low", "adl:hlcv"),
            edge("close", "adl:hlcv"),
            edge("volume", "adl:hlcv"),
            edge("adl:hlcv", indicator_node),
        ],
        IndicatorKind::KST => vec![
            edge("close", "roc:close:10"),
            edge("close", "roc:close:15"),
            edge("close", "roc:close:20"),
            edge("close", "roc:close:30"),
            edge("roc:close:10", "kst:value"),
            edge("roc:close:15", "kst:value"),
            edge("roc:close:20", "kst:value"),
            edge("roc:close:30", "kst:value"),
            edge("kst:value", indicator_node),
            edge("kst:value", &format!("kst:signal:{}", indicator.signal)),
            edge(&format!("kst:signal:{}", indicator.signal), indicator_node),
        ],
        IndicatorKind::BOP => {
            let sma = format!("bop:ohlc:sma:{}", indicator.period);
            vec![
                edge("open", "bop:ohlc"),
                edge("high", "bop:ohlc"),
                edge("low", "bop:ohlc"),
                edge("close", "bop:ohlc"),
                edge("bop:ohlc", &sma),
                edge(&sma, indicator_node),
            ]
        }
        IndicatorKind::ULTIMATE_OSCILLATOR => {
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
        IndicatorKind::CHAIKIN_OSCILLATOR => {
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
        IndicatorKind::FORCE_INDEX => {
            let f = format!("force_sma:close:volume:{}", indicator.period);
            vec![
                edge("close", &f),
                edge("volume", &f),
                edge(&f, indicator_node),
            ]
        }
        IndicatorKind::KELTNER => {
            let ema = format!("ema_sma:close:{}", indicator.period);
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
        IndicatorKind::DONCHIAN => {
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
        IndicatorKind::STOCH_RSI => {
            let r = format!("rsi:close:{}", indicator.period);
            let s = format!(
                "stoch:rsi:{}:{}:{}:{}",
                indicator.period, indicator.stoch_period, indicator.smooth, indicator.signal
            );
            vec![edge("close", &r), edge(&r, &s), edge(&s, indicator_node)]
        }
        IndicatorKind::ADX => {
            let a = format!("adx:ohlc:{}", indicator.period);
            vec![
                edge("high", &a),
                edge("low", &a),
                edge("close", &a),
                edge(&a, indicator_node),
            ]
        }
        IndicatorKind::VWAP => vec![
            edge("high", "vwap:hlcv"),
            edge("low", "vwap:hlcv"),
            edge("close", "vwap:hlcv"),
            edge("volume", "vwap:hlcv"),
            edge("vwap:hlcv", indicator_node),
        ],
        IndicatorKind::PPO => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let f = format!("ema_sma:close:{}", p.fast);
            let s = format!("ema_sma:close:{}", p.slow);
            let ppo = format!("ppo:{}:{}:{}", p.fast, p.slow, p.signal);
            vec![
                edge("close", &f),
                edge("close", &s),
                edge(&f, &ppo),
                edge(&s, &ppo),
                edge(&ppo, indicator_node),
            ]
        }
        IndicatorKind::STOCHASTIC => {
            let s = format!(
                "stoch:hlc:{}:{}:{}",
                indicator.period, indicator.smooth, indicator.signal
            );
            vec![
                edge("high", &s),
                edge("low", &s),
                edge("close", &s),
                edge(&s, indicator_node),
            ]
        }
        IndicatorKind::MEDIAN_PRICE => {
            let m = format!("median_price:hl:{}", indicator.period);
            vec![edge("hl2", &m), edge(&m, indicator_node)]
        }
        IndicatorKind::HIGHEST_HIGH => {
            let h = format!("highest_high:h:{}", indicator.period);
            vec![edge("high", &h), edge(&h, indicator_node)]
        }
        IndicatorKind::LOWEST_LOW => {
            let l = format!("lowest_low:l:{}", indicator.period);
            vec![edge("low", &l), edge(&l, indicator_node)]
        }
        IndicatorKind::ALLIGATOR => vec![
            edge("high", "alligator_seeded:jaw"),
            edge("low", "alligator_seeded:jaw"),
            edge("high", "alligator_seeded:teeth"),
            edge("low", "alligator_seeded:teeth"),
            edge("high", "alligator_seeded:lips"),
            edge("low", "alligator_seeded:lips"),
            edge("alligator_seeded:jaw", indicator_node),
            edge("alligator_seeded:teeth", indicator_node),
            edge("alligator_seeded:lips", indicator_node),
        ],
        IndicatorKind::ATR_BANDS => {
            let atr = format!("atr:ohlc:{}", indicator.period);
            let u = format!(
                "atr_bands:upper:{}:{}",
                indicator.period, indicator.multiplier
            );
            let l = format!(
                "atr_bands:lower:{}:{}",
                indicator.period, indicator.multiplier
            );
            vec![
                edge("high", &atr),
                edge("low", &atr),
                edge("close", &atr),
                edge("close", &u),
                edge(&atr, &u),
                edge("close", &l),
                edge(&atr, &l),
                edge(&u, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        IndicatorKind::HIGH_LOW_BANDS => {
            let m = format!("hlbands:middle:{}", indicator.period);
            let u = format!(
                "hlbands:upper:{}:{}",
                indicator.period, indicator.multiplier
            );
            let l = format!(
                "hlbands:lower:{}:{}",
                indicator.period, indicator.multiplier
            );
            vec![
                edge("close", &m),
                edge(&m, &u),
                edge(&m, &l),
                edge(&u, indicator_node),
                edge(&m, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        IndicatorKind::FRACTAL_CHAOS_BANDS => vec![
            edge("high", "fcb_ties:upper"),
            edge("low", "fcb_ties:upper"),
            edge("high", "fcb_ties:lower"),
            edge("low", "fcb_ties:lower"),
            edge("fcb_ties:upper", indicator_node),
            edge("fcb_ties:lower", indicator_node),
        ],
        IndicatorKind::GMMA => {
            let mut edges = Vec::new();
            for &p in &[3, 5, 8, 10, 12, 15, 30, 35, 40, 45, 50, 60] {
                let e = format!("ema_sma:close:{p}");
                edges.push(edge("close", &e));
                edges.push(edge(&e, indicator_node));
            }
            edges
        }
        IndicatorKind::LINEAR_REG_FORECAST => {
            let l = format!("linreg_forecast_current:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        IndicatorKind::LINEAR_REG_INTERCEPT => {
            let l = format!("linreg_intercept:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        IndicatorKind::ANCHORED_VWAP => {
            let a = format!("anchored_vwap:{}", indicator.anchor);
            vec![
                edge("high", &a),
                edge("low", &a),
                edge("close", &a),
                edge("volume", &a),
                edge(&a, indicator_node),
            ]
        }
        IndicatorKind::TYPICAL_PRICE => {
            let t = format!("typical_price:hlc:{}", indicator.period);
            vec![edge("hlc3", &t), edge(&t, indicator_node)]
        }
        IndicatorKind::WEIGHTED_CLOSE => {
            let w = format!("weighted_close:hlc:{}", indicator.period);
            vec![
                edge("high", &w),
                edge("low", &w),
                edge("close", &w),
                edge(&w, indicator_node),
            ]
        }
        IndicatorKind::MA_CROSS => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: indicator.period,
                slow: indicator.stoch_period,
                signal: 9,
            });
            let f = format!("sma:close:{}", p.fast);
            let s = format!("sma:close:{}", p.slow);
            vec![
                edge("close", &f),
                edge("close", &s),
                edge(&f, indicator_node),
                edge(&s, indicator_node),
            ]
        }
        IndicatorKind::RAINBOW_MA => {
            let mut edges = vec![edge("close", &format!("rainbow:r1:{}", indicator.period))];
            for i in 1..10 {
                edges.push(edge(
                    &format!("rainbow:r{}:{}", i, indicator.period),
                    &format!("rainbow:r{}:{}", i + 1, indicator.period),
                ));
            }
            for i in 1..=10 {
                edges.push(edge(
                    &format!("rainbow:r{}:{}", i, indicator.period),
                    indicator_node,
                ));
            }
            edges
        }
        IndicatorKind::PRIME_NUMBER_BANDS => vec![
            edge("high", "pnb_strict:upper"),
            edge("low", "pnb_strict:lower"),
            edge("pnb_strict:upper", indicator_node),
            edge("pnb_strict:lower", indicator_node),
        ],
        IndicatorKind::TIME_SERIES_FORECAST => {
            let l = format!("tsf:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        IndicatorKind::VALUATION_LINES => {
            let sma = format!("sma:close:{}", indicator.period);
            let u = format!(
                "valuation:upper:{}:{}",
                indicator.period, indicator.multiplier
            );
            let l = format!(
                "valuation:lower:{}:{}",
                indicator.period, indicator.multiplier
            );
            vec![
                edge("close", &sma),
                edge(&sma, &u),
                edge(&sma, &l),
                edge(&u, indicator_node),
                edge(&sma, indicator_node),
                edge(&l, indicator_node),
            ]
        }
        IndicatorKind::BETA => {
            let b = format!("beta:close:{}", indicator.period);
            vec![edge("close", &b), edge(&b, indicator_node)]
        }
        IndicatorKind::CORRELATION_COEFFICIENT => {
            let c = format!("correl:close:{}", indicator.period);
            vec![edge("close", &c), edge(&c, indicator_node)]
        }
        IndicatorKind::PERFORMANCE_INDEX => vec![
            edge("close", "perf_index:close"),
            edge("perf_index:close", indicator_node),
        ],
        IndicatorKind::PRICE_RELATIVE => {
            let p = format!("price_relative:close:{}", indicator.period);
            vec![edge("close", &p), edge(&p, indicator_node)]
        }
        IndicatorKind::AWESOME_OSCILLATOR => vec![
            edge("high", "ao:hl"),
            edge("low", "ao:hl"),
            edge("ao:hl", indicator_node),
        ],
        IndicatorKind::BOLLINGER_PCT_B => {
            let b = format!("bb_pctb100:{}:{}", indicator.period, indicator.multiplier);
            vec![edge("close", &b), edge(&b, indicator_node)]
        }
        IndicatorKind::CENTER_OF_GRAVITY => {
            let c = format!("cog_ehlers:close:{}", indicator.period);
            vec![edge("close", &c), edge(&c, indicator_node)]
        }
        IndicatorKind::CHANDE_FORECAST => {
            let c = format!("cfo:close:{}", indicator.period);
            vec![edge("close", &c), edge(&c, indicator_node)]
        }
        IndicatorKind::CHANDE_MOMENTUM => {
            let c = format!("cmo:close:{}", indicator.period);
            vec![edge("close", &c), edge(&c, indicator_node)]
        }
        IndicatorKind::COPPOCK_CURVE => vec![
            edge("close", "coppock:close"),
            edge("coppock:close", indicator_node),
        ],
        IndicatorKind::DISPARITY_INDEX => {
            let e = format!("sma:close:{}", indicator.period);
            let d = format!("disparity_sma:close:{}", indicator.period);
            vec![edge("close", &e), edge(&e, &d), edge(&d, indicator_node)]
        }
        IndicatorKind::EASE_OF_MOVEMENT => {
            let e = format!("emv_scaled:hlv:{}", indicator.period);
            vec![
                edge("high", &e),
                edge("low", &e),
                edge("volume", &e),
                edge(&e, indicator_node),
            ]
        }
        IndicatorKind::EHLER_FISHER => {
            let f = format!("fisher_mid:hl:{}", indicator.period);
            vec![edge("hl2", &f), edge(&f, indicator_node)]
        }
        IndicatorKind::ELDER_RAY => {
            let e = format!("ema_sma:close:{}", indicator.period);
            let r = format!("elder_ray:hl:{}", indicator.period);
            vec![
                edge("close", &e),
                edge("high", &r),
                edge("low", &r),
                edge(&e, &r),
                edge(&r, indicator_node),
            ]
        }
        IndicatorKind::FRACTAL_CHAOS_OSCILLATOR => vec![
            edge("high", "fco_ties:hl"),
            edge("low", "fco_ties:hl"),
            edge("fco_ties:hl", indicator_node),
        ],
        IndicatorKind::GATOR_OSCILLATOR => {
            let mut edges = Vec::new();
            for line in ["jaw", "teeth", "lips"] {
                let node = format!("alligator_seeded:{line}");
                edges.push(edge("high", &node));
                edges.push(edge("low", &node));
                edges.push(edge(&node, "gator_drawn:upper"));
                edges.push(edge(&node, "gator_drawn:lower"));
            }
            edges.push(edge("gator_drawn:upper", indicator_node));
            edges.push(edge("gator_drawn:lower", indicator_node));
            edges
        }
        IndicatorKind::INTRADAY_MOMENTUM => {
            let i = format!("imi:oc:{}", indicator.period);
            vec![edge("close", &i), edge(&i, indicator_node)]
        }
        IndicatorKind::LINEAR_REG_SLOPE => {
            let l = format!("linreg_slope:close:{}", indicator.period);
            vec![edge("close", &l), edge(&l, indicator_node)]
        }
        IndicatorKind::MA_DEVIATION => {
            let s = format!("sma:close:{}", indicator.period);
            let d = format!("ma_dev:close:{}", indicator.period);
            vec![edge("close", &s), edge(&s, &d), edge(&d, indicator_node)]
        }
        IndicatorKind::PRETTY_GOOD_OSCILLATOR => {
            let s = format!("sma:close:{}", indicator.period);
            let a = "true_range:hlc".to_string();
            let p = format!("pgo_ema_tr:close:{}", indicator.period);
            vec![
                edge("close", &s),
                edge("high", &a),
                edge("low", &a),
                edge("close", &a),
                edge(&s, &p),
                edge(&a, &p),
                edge(&p, indicator_node),
            ]
        }
        IndicatorKind::PRICE_MOMENTUM_OSCILLATOR => {
            let p = format!("pmo_dp:close:{}:{}", indicator.period, indicator.smooth);
            vec![edge("close", &p), edge(&p, indicator_node)]
        }
        IndicatorKind::PRICE_OSCILLATOR => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let p = format!("price_osc_pts:close:{}:{}", params.fast, params.slow);
            vec![
                edge("close", &format!("ema_sma:close:{}", params.fast)),
                edge("close", &format!("ema_sma:close:{}", params.slow)),
                edge(&p, indicator_node),
            ]
        }
        IndicatorKind::RAINBOW_OSCILLATOR => {
            let r = format!(
                "rainbow_osc_range:close:{}:{}",
                indicator.period, indicator.smooth
            );
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        IndicatorKind::RAVI => {
            let r = format!("ravi:close:{}:{}", indicator.period, indicator.stoch_period);
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        IndicatorKind::RELATIVE_VIGOR => {
            let r = format!("rvi:ohlc:{}", indicator.period);
            vec![
                edge("high", &r),
                edge("low", &r),
                edge("close", &r),
                edge(&r, indicator_node),
            ]
        }
        IndicatorKind::SCHAFF_TREND_CYCLE => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let s = format!(
                "stc_seeded:{}:{}:{}",
                p.fast, p.slow, indicator.stoch_period
            );
            vec![edge("close", &s), edge(&s, indicator_node)]
        }
        IndicatorKind::STOCHASTIC_MOMENTUM => {
            let s = format!("smi_sma:hlc:{}:{}", indicator.period, indicator.smooth);
            vec![
                edge("high", &s),
                edge("low", &s),
                edge("close", &s),
                edge(&s, indicator_node),
            ]
        }
        IndicatorKind::SWING_INDEX => {
            let s = format!("swing_index_wilder:ohlc:{}", indicator.multiplier);
            vec![
                edge("open", &s),
                edge("high", &s),
                edge("low", &s),
                edge("close", &s),
                edge(&s, indicator_node),
            ]
        }
        IndicatorKind::TREND_INTENSITY => {
            let s = format!("sma:close:{}", indicator.period);
            let t = format!("tii:close:{}", indicator.period);
            vec![edge("close", &s), edge(&s, &t), edge(&t, indicator_node)]
        }
        IndicatorKind::VOLUME_OSCILLATOR => {
            let p = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let v = format!("vol_osc_pts:volume:{}:{}", p.fast, p.slow);
            vec![edge("volume", &v), edge(&v, indicator_node)]
        }
        IndicatorKind::KLINGER_VOLUME => {
            let p = indicator.macd.unwrap_or(KLINGER_DEFAULTS);
            let kvo = klinger_node(p);
            let signal = format!("klinger:signal:{}:{}:{}", p.fast, p.slow, p.signal);
            vec![
                edge("high", &kvo),
                edge("low", &kvo),
                edge("close", &kvo),
                edge("volume", &kvo),
                edge(&kvo, indicator_node),
                edge(&kvo, &signal),
                edge(&signal, indicator_node),
            ]
        }
        IndicatorKind::MARKET_FACILITATION => {
            let m = format!("mfi_bw:hlv:{}", indicator.multiplier);
            vec![
                edge("high", &m),
                edge("low", &m),
                edge("volume", &m),
                edge(&m, indicator_node),
            ]
        }
        IndicatorKind::NEGATIVE_VOLUME_INDEX => {
            let sma = format!("nvi100:cv:sma:{}", indicator.period);
            vec![
                edge("close", "nvi100:cv"),
                edge("volume", "nvi100:cv"),
                edge("nvi100:cv", indicator_node),
                edge("nvi100:cv", &sma),
                edge(&sma, indicator_node),
            ]
        }
        IndicatorKind::POSITIVE_VOLUME_INDEX => {
            let sma = format!("pvi100:cv:sma:{}", indicator.period);
            vec![
                edge("close", "pvi100:cv"),
                edge("volume", "pvi100:cv"),
                edge("pvi100:cv", indicator_node),
                edge("pvi100:cv", &sma),
                edge(&sma, indicator_node),
            ]
        }
        IndicatorKind::PRICE_VOLUME_TREND => vec![
            edge("close", "pvt:cv"),
            edge("volume", "pvt:cv"),
            edge("pvt:cv", indicator_node),
        ],
        IndicatorKind::TRADE_VOLUME_INDEX => vec![
            edge("close", "tvi:cv"),
            edge("volume", "tvi:cv"),
            edge("tvi:cv", indicator_node),
        ],
        IndicatorKind::TWIGGS_MONEY_FLOW => {
            let t = format!("tmf:hlcv:{}", indicator.period);
            vec![
                edge("high", &t),
                edge("low", &t),
                edge("close", &t),
                edge("volume", &t),
                edge(&t, indicator_node),
            ]
        }
        IndicatorKind::PROJECTED_AGGREGATE_VOLUME => {
            let p = format!("pav:v:{}", indicator.period);
            vec![edge("volume", &p), edge(&p, indicator_node)]
        }
        IndicatorKind::PROJECTED_VOLUME_AT_TIME => {
            let p = format!("pvat:v:{}", indicator.period);
            vec![edge("volume", &p), edge(&p, indicator_node)]
        }
        IndicatorKind::HISTORICAL_VOLATILITY => {
            let h = format!("hv_pop:close:{}", indicator.period);
            vec![edge("close", &h), edge(&h, indicator_node)]
        }
        IndicatorKind::LINEAR_REG_R2 => {
            let r = format!("linreg_r2:close:{}", indicator.period);
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        IndicatorKind::PRIME_NUMBER_OSCILLATOR => {
            let p = format!("pno_bands:hlc:{}", indicator.multiplier);
            vec![
                edge("high", &p),
                edge("low", &p),
                edge("close", &p),
                edge(&p, indicator_node),
            ]
        }
        IndicatorKind::RANDOM_WALK_INDEX => {
            let r = format!("rwi:hlc:{}", indicator.period);
            vec![
                edge("high", &r),
                edge("low", &r),
                edge("close", &r),
                edge(&r, indicator_node),
            ]
        }
        IndicatorKind::DARVAS_BOX => vec![
            edge("high", "darvas:top"),
            edge("low", "darvas:bottom"),
            edge("darvas:top", indicator_node),
            edge("darvas:bottom", indicator_node),
        ],
        IndicatorKind::VOLUME_PROFILE => {
            let v = format!("vol_profile:hlv:{}", indicator.period);
            vec![
                edge("high", &v),
                edge("low", &v),
                edge("volume", &v),
                edge(&v, indicator_node),
            ]
        }
        IndicatorKind::CHOPPINESS_INDEX => {
            let c = format!("chop:hlc:{}", indicator.period);
            vec![
                edge("high", &c),
                edge("low", &c),
                edge("close", &c),
                edge(&c, indicator_node),
            ]
        }
        IndicatorKind::ELDER_IMPULSE => {
            let e = format!("impulse:close:{}", indicator.period);
            vec![edge("close", &e), edge(&e, indicator_node)]
        }
        IndicatorKind::GONOGO_TREND => {
            let g = format!("gonogo:close:{}", indicator.period);
            vec![edge("close", &g), edge(&g, indicator_node)]
        }
        IndicatorKind::PSYCHOLOGICAL_LINE => {
            let p = format!("psy:close:{}", indicator.period);
            vec![edge("close", &p), edge(&p, indicator_node)]
        }
        IndicatorKind::QSTICK => {
            let q = format!("qstick:oc:{}", indicator.period);
            vec![edge("close", &q), edge(&q, indicator_node)]
        }
        IndicatorKind::SHINOHARA_INTENSITY => {
            let s = format!("shinohara:hlc:{}", indicator.period);
            vec![
                edge("high", &s),
                edge("low", &s),
                edge("close", &s),
                edge(&s, indicator_node),
            ]
        }
        IndicatorKind::ULCER_INDEX => {
            let u = format!("ulcer:close:{}", indicator.period);
            vec![edge("close", &u), edge(&u, indicator_node)]
        }
        IndicatorKind::VERTICAL_HORIZONTAL_FILTER => {
            let v = format!("vhf:close:{}", indicator.period);
            vec![edge("close", &v), edge(&v, indicator_node)]
        }
        IndicatorKind::VORTEX_INDICATOR => {
            let v = format!("vortex:hlc:{}", indicator.period);
            vec![
                edge("high", &v),
                edge("low", &v),
                edge("close", &v),
                edge(&v, indicator_node),
            ]
        }
        IndicatorKind::ZIGZAG => {
            let z = format!("zigzag:hl:{}", indicator.multiplier);
            vec![edge("high", &z), edge("low", &z), edge(&z, indicator_node)]
        }
        IndicatorKind::BOLLINGER_BANDWIDTH => {
            let b = format!("bb_bw:{}:{}", indicator.period, indicator.multiplier);
            vec![edge("close", &b), edge(&b, indicator_node)]
        }
        IndicatorKind::DONCHIAN_WIDTH => {
            let d = format!("donchian_width_points:{}", indicator.period);
            vec![edge("high", &d), edge("low", &d), edge(&d, indicator_node)]
        }
        IndicatorKind::GOPALAKRISHNAN_RANGE => {
            let g = format!("gapo:hl:{}", indicator.period);
            vec![edge("high", &g), edge("low", &g), edge(&g, indicator_node)]
        }
        IndicatorKind::HIGH_MINUS_LOW => vec![
            edge("high", "hml:hl"),
            edge("low", "hml:hl"),
            edge("hml:hl", indicator_node),
        ],
        IndicatorKind::MASS_INDEX => {
            let m = format!("mass_sma:hl:{}", indicator.period);
            vec![edge("high", &m), edge("low", &m), edge(&m, indicator_node)]
        }
        IndicatorKind::RELATIVE_VOLATILITY => {
            let r = format!("rvi_vol:close:{}", indicator.period);
            vec![edge("close", &r), edge(&r, indicator_node)]
        }
        IndicatorKind::TRUE_RANGE => vec![
            edge("high", "true_range:hlc"),
            edge("low", "true_range:hlc"),
            edge("close", "true_range:hlc"),
            edge("true_range:hlc", indicator_node),
        ],
        IndicatorKind::VOLUME_CHART => vec![
            edge("volume", "vol_chart:v"),
            edge("vol_chart:v", indicator_node),
        ],
        IndicatorKind::VOLUME_ROC => {
            let v = format!("vol_roc:v:{}", indicator.period);
            vec![edge("volume", &v), edge(&v, indicator_node)]
        }
        IndicatorKind::VOLUME_UNDERLAY => vec![
            edge("close", "vol_underlay:cv"),
            edge("volume", "vol_underlay:cv"),
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
    kind: IndicatorKind,
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
    if kind.uses_macd_params() {
        let macd = macd.expect("MACD params are built before validation");
        if macd.fast == 0 || macd.slow <= macd.fast || macd.signal == 0 {
            return Err(wasm_bindgen::JsValue::from_str(
                "fast/slow params must satisfy fast > 0, slow > fast, signal > 0",
            ));
        }
    } else if kind == IndicatorKind::ICHIMOKU
        && (tenkan_period == 0
            || kijun_period == 0
            || senkou_b_period == 0
            || kijun_period < tenkan_period
            || senkou_b_period < kijun_period)
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "ICHIMOKU params must satisfy tenkan > 0, kijun >= tenkan, senkou_b >= kijun",
        ));
    } else if kind == IndicatorKind::PARABOLIC_SAR
        && (!psar_step.is_finite()
            || !psar_max_step.is_finite()
            || psar_step <= 0.0
            || psar_max_step <= 0.0
            || psar_max_step < psar_step)
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "PARABOLIC_SAR params must satisfy step > 0 and max_step >= step",
        ));
    } else if kind.needs_period() && period == 0 {
        return Err(wasm_bindgen::JsValue::from_str(
            "period must be greater than zero",
        ));
    } else if matches!(kind, IndicatorKind::STOCH_RSI | IndicatorKind::TSI) && stoch_period == 0 {
        return Err(wasm_bindgen::JsValue::from_str(
            "stoch_period must be greater than zero",
        ));
    } else if matches!(
        kind,
        IndicatorKind::STOCHASTIC
            | IndicatorKind::STOCH_RSI
            | IndicatorKind::CHAIKIN_VOLATILITY
            | IndicatorKind::STARC
    ) && smooth == 0
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "smooth must be greater than zero",
        ));
    } else if matches!(
        kind,
        IndicatorKind::STOCHASTIC | IndicatorKind::STOCH_RSI | IndicatorKind::KST
    ) && signal == 0
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "signal must be greater than zero",
        ));
    } else if kind == IndicatorKind::ULTIMATE_OSCILLATOR
        && (stoch_period < period || smooth < stoch_period)
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "ULTIMATE_OSCILLATOR params must satisfy short <= medium <= long",
        ));
    } else if matches!(
        kind,
        IndicatorKind::BB
            | IndicatorKind::SUPERTREND
            | IndicatorKind::KELTNER
            | IndicatorKind::ENVELOPE
            | IndicatorKind::STARC
            | IndicatorKind::ATR_BANDS
            | IndicatorKind::HIGH_LOW_BANDS
    ) && (!multiplier.is_finite() || multiplier <= 0.0)
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "multiplier must be greater than zero",
        ));
    } else if kind == IndicatorKind::SWING_INDEX && !(multiplier.is_finite() && multiplier > 0.0) {
        return Err(wasm_bindgen::JsValue::from_str(
            "SWING_INDEX limit move must be greater than zero",
        ));
    } else if kind == IndicatorKind::RAINBOW_OSCILLATOR && smooth == 0 {
        return Err(wasm_bindgen::JsValue::from_str(
            "RAINBOW_OSCILLATOR lookback must be greater than zero",
        ));
    } else if kind == IndicatorKind::PRICE_MOMENTUM_OSCILLATOR && (period < 2 || smooth < 2) {
        return Err(wasm_bindgen::JsValue::from_str(
            "PRICE_MOMENTUM_OSCILLATOR periods must be at least 2",
        ));
    } else if kind == IndicatorKind::PRIME_NUMBER_OSCILLATOR
        && (!multiplier.is_finite() || multiplier < 0.0)
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "PRIME_NUMBER_OSCILLATOR tolerance must be zero or more",
        ));
    }
    Ok(())
}

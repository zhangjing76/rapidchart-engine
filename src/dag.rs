use crate::types::{DagEdge, Indicator, MacdParams};

pub(crate) fn supports_incremental(kind: &str) -> bool {
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
        _ => indicator_nodes(indicator)
            .into_iter()
            .map(|node| edge(&node, indicator_node))
            .collect(),
    }
}

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
    if kind == "MACD" || kind == "PPO" || kind == "CHAIKIN_OSCILLATOR" {
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
    } else if kind != "OBV"
        && kind != "VWAP"
        && kind != "PARABOLIC_SAR"
        && kind != "ICHIMOKU"
        && kind != "PIVOT_POINTS"
        && kind != "ADL"
        && kind != "WILLIAMS_AD"
        && kind != "DEMA"
        && kind != "TEMA"
        && kind != "TRIMA"
        && kind != "STDDEV"
        && kind != "ENVELOPE"
        && kind != "KST"
        && kind != "BOP"
        && period == 0
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "period must be greater than zero",
        ));
    } else if kind == "STOCH_RSI" && stoch_period == 0 {
        return Err(wasm_bindgen::JsValue::from_str(
            "stoch_period must be greater than zero",
        ));
    } else if kind == "TSI" && stoch_period == 0 {
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
        || kind == "STARC")
        && (!multiplier.is_finite() || multiplier <= 0.0)
    {
        return Err(wasm_bindgen::JsValue::from_str(
            "multiplier must be greater than zero",
        ));
    }
    Ok(())
}

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct IndicatorDescriptor {
    pub kind: &'static str,
    pub name: &'static str,
    pub pane: &'static str,
    pub params: Vec<ParamDescriptor>,
    pub outputs: Vec<OutputDescriptor>,
}

#[derive(Serialize)]
pub(crate) struct ParamDescriptor {
    pub name: &'static str,
    pub label: &'static str,
    pub default: f64,
    pub min: f64,
    pub step: &'static str,
}

#[derive(Serialize)]
pub(crate) struct OutputDescriptor {
    pub name: &'static str,
    pub renderer: &'static str,
    pub pane: &'static str,
    pub color: &'static str,
}

pub(crate) fn period_descriptor(
    kind: &'static str,
    name: &'static str,
    pane: &'static str,
    default: usize,
) -> IndicatorDescriptor {
    IndicatorDescriptor {
        kind,
        name,
        pane,
        params: vec![ParamDescriptor {
            name: "period",
            label: "Period",
            default: default as f64,
            min: 1.0,
            step: "1",
        }],
        outputs: vec![output_descriptor("value", "line", pane, "#2563eb")],
    }
}

pub(crate) fn output_descriptor(
    name: &'static str,
    renderer: &'static str,
    pane: &'static str,
    color: &'static str,
) -> OutputDescriptor {
    OutputDescriptor {
        name,
        renderer,
        pane,
        color,
    }
}

pub(crate) fn indicator_descriptors() -> Vec<IndicatorDescriptor> {
    vec![
        period_descriptor("SMA", "SMA", "overlay", 20),
        period_descriptor("EMA", "EMA", "overlay", 20),
        period_descriptor("WMA", "WMA", "overlay", 20),
        period_descriptor("HMA", "HMA", "overlay", 20),
        period_descriptor("LINEAR_REGRESSION", "LINEAR REGRESSION", "overlay", 20),
        period_descriptor("DEMA", "DEMA", "overlay", 20),
        period_descriptor("TEMA", "TEMA", "overlay", 20),
        period_descriptor("TRIMA", "TRIMA", "overlay", 20),
        period_descriptor("STDDEV", "STDDEV", "separate", 20),
        period_descriptor("TRIX", "TRIX", "separate", 15),
        IndicatorDescriptor {
            kind: "ENVELOPE",
            name: "ENVELOPE",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 20.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier %",
                    default: 2.0,
                    min: 0.1,
                    step: "0.1",
                },
            ],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#2563eb"),
                output_descriptor("middle", "line", "overlay", "#64748b"),
                output_descriptor("lower", "line", "overlay", "#dc2626"),
            ],
        },
        IndicatorDescriptor {
            kind: "TSI",
            name: "TSI",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Long",
                    default: 25.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "stoch_period",
                    label: "Short",
                    default: 13.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "separate", "#2563eb")],
        },
        period_descriptor("DPO", "DPO", "separate", 20),
        period_descriptor("MOMENTUM", "MOMENTUM", "separate", 10),
        period_descriptor("RSI", "RSI", "separate", 14),
        period_descriptor("ROC", "ROC", "separate", 14),
        period_descriptor("CCI", "CCI", "separate", 20),
        period_descriptor("MFI", "MFI", "separate", 14),
        period_descriptor("CMF", "CMF", "separate", 20),
        period_descriptor("FORCE_INDEX", "FORCE INDEX", "separate", 13),
        period_descriptor("VWMA", "VWMA", "overlay", 20),
        IndicatorDescriptor {
            kind: "WILLIAMS_AD",
            name: "WILLIAMS A/D",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#9333ea")],
        },
        period_descriptor("CHAIKIN_VOLATILITY", "CHAIKIN VOLATILITY", "separate", 10),
        IndicatorDescriptor {
            kind: "PRICE_CHANNEL",
            name: "PRICE CHANNEL",
            pane: "overlay",
            params: vec![ParamDescriptor {
                name: "period",
                label: "Period",
                default: 20.0,
                min: 1.0,
                step: "1",
            }],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#f59e0b"),
                output_descriptor("middle", "line", "overlay", "#64748b"),
                output_descriptor("lower", "line", "overlay", "#f59e0b"),
            ],
        },
        IndicatorDescriptor {
            kind: "STARC",
            name: "STARC",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 15.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier",
                    default: 2.0,
                    min: 0.1,
                    step: "0.1",
                },
            ],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#0f766e"),
                output_descriptor("middle", "line", "overlay", "#2563eb"),
                output_descriptor("lower", "line", "overlay", "#0f766e"),
            ],
        },
        period_descriptor("WILLIAMS_R", "WILLIAMS %R", "separate", 14),
        IndicatorDescriptor {
            kind: "PARABOLIC_SAR",
            name: "PARABOLIC SAR",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "psar_step",
                    label: "Step",
                    default: 0.02,
                    min: 0.001,
                    step: "0.001",
                },
                ParamDescriptor {
                    name: "psar_max_step",
                    label: "Max",
                    default: 0.2,
                    min: 0.01,
                    step: "0.01",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "overlay", "#059669")],
        },
        IndicatorDescriptor {
            kind: "ICHIMOKU",
            name: "ICHIMOKU",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "tenkan_period",
                    label: "Tenkan",
                    default: 9.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "kijun_period",
                    label: "Kijun",
                    default: 26.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "senkou_b_period",
                    label: "Senkou B",
                    default: 52.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("tenkan", "line", "overlay", "#2563eb"),
                output_descriptor("kijun", "line", "overlay", "#dc2626"),
                output_descriptor("senkou_a", "line", "overlay", "#059669"),
                output_descriptor("senkou_b", "line", "overlay", "#ea580c"),
                output_descriptor("chikou", "line", "overlay", "#64748b"),
            ],
        },
        IndicatorDescriptor {
            kind: "PIVOT_POINTS",
            name: "PIVOT POINTS",
            pane: "overlay",
            params: Vec::new(),
            outputs: vec![
                output_descriptor("pp", "line", "overlay", "#64748b"),
                output_descriptor("r1", "line", "overlay", "#059669"),
                output_descriptor("s1", "line", "overlay", "#dc2626"),
                output_descriptor("r2", "line", "overlay", "#16a34a"),
                output_descriptor("s2", "line", "overlay", "#b91c1c"),
            ],
        },
        IndicatorDescriptor {
            kind: "AROON",
            name: "AROON",
            pane: "separate",
            params: vec![ParamDescriptor {
                name: "period",
                label: "Period",
                default: 14.0,
                min: 1.0,
                step: "1",
            }],
            outputs: vec![
                output_descriptor("up", "line", "separate", "#059669"),
                output_descriptor("down", "line", "separate", "#dc2626"),
                output_descriptor("oscillator", "line", "separate", "#2563eb"),
            ],
        },
        IndicatorDescriptor {
            kind: "ADL",
            name: "ADL",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#9333ea")],
        },
        IndicatorDescriptor {
            kind: "KST",
            name: "KST",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#2563eb")],
        },
        IndicatorDescriptor {
            kind: "BOP",
            name: "BOP",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#9333ea")],
        },
        IndicatorDescriptor {
            kind: "ULTIMATE_OSCILLATOR",
            name: "ULTIMATE OSCILLATOR",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Short",
                    default: 7.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "stoch_period",
                    label: "Medium",
                    default: 14.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "smooth",
                    label: "Long",
                    default: 28.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "separate", "#2563eb")],
        },
        IndicatorDescriptor {
            kind: "SUPERTREND",
            name: "SUPERTREND",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 10.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier",
                    default: 3.0,
                    min: 1.0,
                    step: "0.1",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "overlay", "#0f766e")],
        },
        IndicatorDescriptor {
            kind: "KELTNER",
            name: "KELTNER",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 20.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier",
                    default: 2.0,
                    min: 1.0,
                    step: "0.1",
                },
            ],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#0f766e"),
                output_descriptor("middle", "line", "overlay", "#2563eb"),
                output_descriptor("lower", "line", "overlay", "#0f766e"),
            ],
        },
        IndicatorDescriptor {
            kind: "DONCHIAN",
            name: "DONCHIAN",
            pane: "overlay",
            params: vec![ParamDescriptor {
                name: "period",
                label: "Period",
                default: 20.0,
                min: 1.0,
                step: "1",
            }],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#f59e0b"),
                output_descriptor("middle", "line", "overlay", "#64748b"),
                output_descriptor("lower", "line", "overlay", "#f59e0b"),
            ],
        },
        IndicatorDescriptor {
            kind: "STOCH_RSI",
            name: "STOCH RSI",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 14.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "stoch_period",
                    label: "Stoch",
                    default: 14.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "smooth",
                    label: "%K",
                    default: 3.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "signal",
                    label: "%D",
                    default: 3.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("k", "line", "separate", "#2563eb"),
                output_descriptor("d", "line", "separate", "#dc2626"),
            ],
        },
        period_descriptor("ATR", "ATR", "separate", 14),
        IndicatorDescriptor {
            kind: "ADX",
            name: "ADX",
            pane: "separate",
            params: vec![ParamDescriptor {
                name: "period",
                label: "Period",
                default: 14.0,
                min: 1.0,
                step: "1",
            }],
            outputs: vec![
                output_descriptor("value", "line", "separate", "#2563eb"),
                output_descriptor("plus_di", "line", "separate", "#059669"),
                output_descriptor("minus_di", "line", "separate", "#dc2626"),
            ],
        },
        IndicatorDescriptor {
            kind: "VWAP",
            name: "VWAP",
            pane: "overlay",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "overlay", "#0f766e")],
        },
        IndicatorDescriptor {
            kind: "STOCHASTIC",
            name: "STOCHASTIC",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 14.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "smooth",
                    label: "Smooth",
                    default: 3.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("k", "line", "separate", "#2563eb"),
                output_descriptor("d", "line", "separate", "#dc2626"),
            ],
        },
        IndicatorDescriptor {
            kind: "OBV",
            name: "OBV",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#059669")],
        },
        IndicatorDescriptor {
            kind: "BB",
            name: "BOLLINGER",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 20.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier",
                    default: 2.0,
                    min: 1.0,
                    step: "0.1",
                },
            ],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#9333ea"),
                output_descriptor("middle", "line", "overlay", "#64748b"),
                output_descriptor("lower", "line", "overlay", "#9333ea"),
            ],
        },
        IndicatorDescriptor {
            kind: "MACD",
            name: "MACD",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "fast",
                    label: "Fast",
                    default: 12.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "slow",
                    label: "Slow",
                    default: 26.0,
                    min: 2.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "signal",
                    label: "Signal",
                    default: 9.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("macd", "line", "separate", "#2563eb"),
                output_descriptor("signal", "line", "separate", "#dc2626"),
                output_descriptor("histogram", "histogram", "separate", "#86efac"),
            ],
        },
        IndicatorDescriptor {
            kind: "PPO",
            name: "PPO",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "fast",
                    label: "Fast",
                    default: 12.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "slow",
                    label: "Slow",
                    default: 26.0,
                    min: 2.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "signal",
                    label: "Signal",
                    default: 9.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("ppo", "line", "separate", "#2563eb"),
                output_descriptor("signal", "line", "separate", "#dc2626"),
                output_descriptor("histogram", "histogram", "separate", "#86efac"),
            ],
        },
        IndicatorDescriptor {
            kind: "CHAIKIN_OSCILLATOR",
            name: "CHAIKIN OSCILLATOR",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "fast",
                    label: "Fast",
                    default: 3.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "slow",
                    label: "Slow",
                    default: 10.0,
                    min: 2.0,
                    step: "1",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "separate", "#9333ea")],
        },
    ]
}

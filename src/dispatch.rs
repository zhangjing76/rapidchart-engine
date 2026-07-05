use crate::bar::{Bar, CandleStore};
use crate::helpers::{materialized_bars, one_output, rc_one_output};
use crate::series::{rc_into_owned, NodeCache};
use crate::types::{IndicatorOutput, MacdParams};

// Import all indicator computation functions
use crate::indicators::adl::{adl_node, adl_store};
use crate::indicators::adx::{adx, adx_store};
use crate::indicators::aroon::{aroon, aroon_store};
use crate::indicators::atr::{atr_node, atr_store};
use crate::indicators::bollinger::{bollinger, bollinger_store};
use crate::indicators::bop::{bop_node, bop_store};
use crate::indicators::cci::{cci_node, cci_store};
use crate::indicators::chaikin::{
    chaikin_oscillator_node, chaikin_oscillator_store, chaikin_volatility_node,
    chaikin_volatility_store,
};
use crate::indicators::cmf::{cmf_node, cmf_store};
use crate::indicators::dema::dema_store;
use crate::indicators::donchian::{donchian, donchian_store, price_channel, price_channel_store};
use crate::indicators::dpo::{dpo_node, dpo_store};
use crate::indicators::ema::{ema_close, ema_close_store};
use crate::indicators::envelope::envelope_store;
use crate::indicators::force_index::{force_index_node, force_index_store};
use crate::indicators::hma::{hma, hma_store};
use crate::indicators::ichimoku::{ichimoku, ichimoku_store};
use crate::indicators::keltner::{keltner, keltner_store, starc, starc_store};
use crate::indicators::kst::{kst_node, kst_store};
use crate::indicators::linear_regression::{linear_regression_node, linear_regression_store};
use crate::indicators::macd::{macd, macd_store, ppo, ppo_store};
use crate::indicators::mfi::{mfi_node, mfi_store};
use crate::indicators::momentum::{momentum_node, momentum_store};
use crate::indicators::obv::{obv_node, obv_store};
use crate::indicators::parabolic_sar::{parabolic_sar, parabolic_sar_store};
use crate::indicators::pivot_points::{pivot_points, pivot_points_store};
use crate::indicators::roc::{roc_node, roc_store};
use crate::indicators::rsi::{rsi_outputs, rsi_outputs_store};
use crate::indicators::sma::{sma_close, sma_close_store};
use crate::indicators::stddev::{stddev_node, stddev_store};
use crate::indicators::stoch::{stochastic, stochastic_store};
use crate::indicators::stoch_rsi::{stoch_rsi, stoch_rsi_store};
use crate::indicators::supertrend::{supertrend, supertrend_store};
use crate::indicators::tema::tema_store;
use crate::indicators::trima::trima_store;
use crate::indicators::trix::{trix_node, trix_store};
use crate::indicators::tsi::{tsi_node, tsi_store};
use crate::indicators::ultimate_oscillator::{ultimate_oscillator_node, ultimate_oscillator_store};
use crate::indicators::vwap::{vwap, vwap_store};
use crate::indicators::vwma::{vwma_node, vwma_store};
use crate::indicators::williams_ad::{williams_ad_node, williams_ad_store};
use crate::indicators::williams_r::{williams_r_node, williams_r_store};
use crate::indicators::wma::{wma_close, wma_store};

pub(crate) fn compute_indicator(
    bars: &[Bar],
    kind: &str,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    macd_params: Option<MacdParams>,
    multiplier: f64,
    psar_step: f64,
    psar_max_step: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    match kind {
        "SMA" => one_output(sma_close(bars, period, nodes)),
        "EMA" => one_output(ema_close(bars, period, nodes)),
        "WMA" => one_output(wma_close(bars, period, nodes)),
        "HMA" => one_output(hma(bars, period, nodes)),
        "LINEAR_REGRESSION" => one_output(linear_regression_node(bars, period, nodes)),
        "TRIX" => one_output(trix_node(bars, period, nodes)),
        "TSI" => one_output(tsi_node(bars, period, stoch_period, nodes)),
        "DPO" => one_output(dpo_node(bars, period, nodes)),
        "MOMENTUM" => one_output(momentum_node(bars, period, nodes)),
        "RSI" => rsi_outputs(bars, period),
        "ROC" => one_output(roc_node(bars, period, nodes)),
        "CCI" => one_output(cci_node(bars, period, nodes)),
        "MFI" => one_output(mfi_node(bars, period, nodes)),
        "CMF" => one_output(cmf_node(bars, period, nodes)),
        "WILLIAMS_R" => one_output(williams_r_node(bars, period, nodes)),
        "PARABOLIC_SAR" => parabolic_sar(bars, psar_step, psar_max_step, nodes),
        "ICHIMOKU" => ichimoku(bars, tenkan_period, kijun_period, senkou_b_period, nodes),
        "PIVOT_POINTS" => pivot_points(bars, nodes),
        "AROON" => aroon(bars, period, nodes),
        "ADL" => one_output(adl_node(bars, nodes)),
        "KST" => one_output(kst_node(bars, nodes)),
        "BOP" => one_output(bop_node(bars, nodes)),
        "ULTIMATE_OSCILLATOR" => one_output(ultimate_oscillator_node(
            bars,
            period,
            stoch_period,
            smooth,
            nodes,
        )),
        "CHAIKIN_OSCILLATOR" => one_output(chaikin_oscillator_node(
            bars,
            macd_params.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            }),
            nodes,
        )),
        "FORCE_INDEX" => one_output(force_index_node(bars, period, nodes)),
        "VWMA" => one_output(vwma_node(bars, period, nodes)),
        "WILLIAMS_AD" => one_output(williams_ad_node(bars, nodes)),
        "CHAIKIN_VOLATILITY" => one_output(chaikin_volatility_node(bars, period, nodes)),
        "PRICE_CHANNEL" => price_channel(bars, period, nodes),
        "STARC" => starc(bars, period, multiplier, nodes),
        "SUPERTREND" => supertrend(bars, period, multiplier, nodes),
        "KELTNER" => keltner(bars, period, multiplier, nodes),
        "DONCHIAN" => donchian(bars, period, nodes),
        "STOCH_RSI" => stoch_rsi(bars, period, stoch_period, smooth, signal, nodes),
        "OBV" => one_output(obv_node(bars, nodes)),
        "ATR" => one_output(atr_node(bars, period, nodes)),
        "ADX" => adx(bars, period, nodes),
        "VWAP" => vwap(bars, nodes),
        "STOCHASTIC" => stochastic(bars, period, smooth, nodes),
        "BB" => bollinger(bars, period, multiplier, nodes),
        "MACD" => macd(
            bars,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        ),
        "PPO" => ppo(
            bars,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        ),
        _ => Vec::new(),
    }
}

pub(crate) fn compute_indicator_store(
    store: &CandleStore,
    kind: &str,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    macd_params: Option<MacdParams>,
    multiplier: f64,
    psar_step: f64,
    psar_max_step: f64,
    nodes: &mut NodeCache,
    bars_snapshot: &mut Option<Vec<Bar>>,
) -> Vec<IndicatorOutput> {
    match kind {
        "SMA" => one_output(rc_into_owned(sma_close_store(store, period, nodes))),
        "EMA" => one_output(rc_into_owned(ema_close_store(store, period, nodes))),
        "RSI" => rsi_outputs_store(store, period, nodes),
        "ROC" => rc_one_output(roc_store(store, period, nodes)),
        "CCI" => rc_one_output(cci_store(store, period, nodes)),
        "MFI" => rc_one_output(mfi_store(store, period, nodes)),
        "CMF" => rc_one_output(cmf_store(store, period, nodes)),
        "WILLIAMS_R" => rc_one_output(williams_r_store(store, period, nodes)),
        "OBV" => one_output(rc_into_owned(obv_store(store, nodes))),
        "ADL" => one_output(rc_into_owned(adl_store(store, nodes))),
        "VWAP" => vwap_store(store, nodes),
        "VWMA" => rc_one_output(vwma_store(store, period, nodes)),
        "WILLIAMS_AD" => rc_one_output(williams_ad_store(store, nodes)),
        "ATR" => one_output(rc_into_owned(atr_store(store, period, nodes))),
        "ADX" => adx_store(store, period, nodes),
        "SUPERTREND" => supertrend_store(store, period, multiplier, nodes),
        "KELTNER" => keltner_store(store, period, multiplier, nodes),
        "STARC" => starc_store(store, period, multiplier, nodes),
        "WMA" => one_output(rc_into_owned(wma_store(store, period, nodes))),
        "HMA" => one_output(rc_into_owned(hma_store(store, period, nodes))),
        "LINEAR_REGRESSION" => rc_one_output(linear_regression_store(store, period, nodes)),
        "DEMA" => rc_one_output(dema_store(store, period, nodes)),
        "TEMA" => rc_one_output(tema_store(store, period, nodes)),
        "TRIMA" => rc_one_output(trima_store(store, period, nodes)),
        "STDDEV" => one_output(rc_into_owned(stddev_store(store, period, nodes))),
        "ENVELOPE" => envelope_store(store, period, multiplier, nodes),
        "TRIX" => rc_one_output(trix_store(store, period, nodes)),
        "TSI" => rc_one_output(tsi_store(store, period, stoch_period, nodes)),
        "KST" => rc_one_output(kst_store(store, nodes)),
        "BOP" => rc_one_output(bop_store(store, nodes)),
        "MOMENTUM" => rc_one_output(momentum_store(store, period, nodes)),
        "DPO" => rc_one_output(dpo_store(store, period, nodes)),
        "FORCE_INDEX" => rc_one_output(force_index_store(store, period, nodes)),
        "PRICE_CHANNEL" => price_channel_store(store, period, nodes),
        "STOCHASTIC" => stochastic_store(store, period, smooth, nodes),
        "BB" => bollinger_store(store, period, multiplier, nodes),
        "DONCHIAN" => donchian_store(store, period, nodes),
        "PARABOLIC_SAR" => parabolic_sar_store(store, psar_step, psar_max_step, nodes),
        "ICHIMOKU" => ichimoku_store(store, tenkan_period, kijun_period, senkou_b_period, nodes),
        "PIVOT_POINTS" => pivot_points_store(store, nodes),
        "AROON" => aroon_store(store, period, nodes),
        "ULTIMATE_OSCILLATOR" => one_output(ultimate_oscillator_store(
            store,
            period,
            stoch_period,
            smooth,
            nodes,
        )),
        "CHAIKIN_VOLATILITY" => rc_one_output(chaikin_volatility_store(store, period, nodes)),
        "STOCH_RSI" => stoch_rsi_store(store, period, stoch_period, smooth, signal, nodes),
        "CHAIKIN_OSCILLATOR" => one_output(chaikin_oscillator_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            }),
            nodes,
        )),
        "MACD" => macd_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        ),
        "PPO" => ppo_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        ),
        _ => compute_indicator(
            materialized_bars(store, bars_snapshot),
            kind,
            period,
            stoch_period,
            smooth,
            signal,
            tenkan_period,
            kijun_period,
            senkou_b_period,
            macd_params,
            multiplier,
            psar_step,
            psar_max_step,
            nodes,
        ),
    }
}

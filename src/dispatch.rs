use crate::bar::CandleStore;
use crate::helpers::rc_one_output;
use crate::series::NodeCache;
use crate::types::{IndicatorOutput, MacdParams};

use crate::indicators::adl::adl_store;
use crate::indicators::adx::adx_store;
use crate::indicators::alligator::alligator_store;
use crate::indicators::anchored_vwap::anchored_vwap_store;
use crate::indicators::aroon::aroon_store;
use crate::indicators::atr::atr_store;
use crate::indicators::atr_bands::atr_bands_store;
use crate::indicators::awesome_oscillator::awesome_oscillator_store;
use crate::indicators::beta::beta_store;
use crate::indicators::bollinger::bollinger_store;
use crate::indicators::bollinger_bandwidth::bollinger_bandwidth_store;
use crate::indicators::bollinger_pct_b::bollinger_pct_b_store;
use crate::indicators::bop::bop_store;
use crate::indicators::cci::cci_store;
use crate::indicators::center_of_gravity::center_of_gravity_store;
use crate::indicators::chaikin::{chaikin_oscillator_store, chaikin_volatility_store};
use crate::indicators::chande_forecast::chande_forecast_store;
use crate::indicators::chande_momentum::chande_momentum_store;
use crate::indicators::choppiness_index::choppiness_index_store;
use crate::indicators::cmf::cmf_store;
use crate::indicators::coppock_curve::coppock_curve_store;
use crate::indicators::correlation_coefficient::correlation_coefficient_store;
use crate::indicators::darvas_box::darvas_box_store;
use crate::indicators::dema::dema_store;
use crate::indicators::disparity_index::disparity_index_store;
use crate::indicators::donchian::{donchian_store, price_channel_store};
use crate::indicators::donchian_width::donchian_width_store;
use crate::indicators::dpo::dpo_store;
use crate::indicators::ease_of_movement::ease_of_movement_store;
use crate::indicators::ehler_fisher::ehler_fisher_store;
use crate::indicators::elder_ray::elder_ray_store;
use crate::indicators::elder_impulse::elder_impulse_store;
use crate::indicators::ema::ema_close_store;
use crate::indicators::envelope::envelope_store;
use crate::indicators::force_index::force_index_store;
use crate::indicators::fractal_chaos_bands::fractal_chaos_bands_store;
use crate::indicators::fractal_chaos_oscillator::fractal_chaos_oscillator_store;
use crate::indicators::gator_oscillator::gator_oscillator_store;
use crate::indicators::gmma::gmma_store;
use crate::indicators::gonogo_trend::gonogo_trend_store;
use crate::indicators::gopalakrishnan_range::gopalakrishnan_range_store;
use crate::indicators::high_low_bands::high_low_bands_store;
use crate::indicators::high_minus_low::high_minus_low_store;
use crate::indicators::highest_high::highest_high_store;
use crate::indicators::historical_volatility::historical_volatility_store;
use crate::indicators::hma::hma_store;
use crate::indicators::ichimoku::ichimoku_store;
use crate::indicators::intraday_momentum::intraday_momentum_store;
use crate::indicators::keltner::{keltner_store, starc_store};
use crate::indicators::klinger_volume::klinger_volume_store;
use crate::indicators::kst::kst_store;
use crate::indicators::linear_reg_forecast::linear_reg_forecast_store;
use crate::indicators::linear_reg_intercept::linear_reg_intercept_store;
use crate::indicators::linear_reg_r2::linear_reg_r2_store;
use crate::indicators::linear_reg_slope::linear_reg_slope_store;
use crate::indicators::linear_regression::linear_regression_store;
use crate::indicators::lowest_low::lowest_low_store;
use crate::indicators::ma_cross::ma_cross_store;
use crate::indicators::ma_deviation::ma_deviation_store;
use crate::indicators::macd::{macd_store, ppo_store};
use crate::indicators::market_facilitation::market_facilitation_store;
use crate::indicators::mass_index::mass_index_store;
use crate::indicators::median_price::median_price_store;
use crate::indicators::mfi::mfi_store;
use crate::indicators::momentum::momentum_store;
use crate::indicators::negative_volume_index::negative_volume_index_store;
use crate::indicators::obv::obv_store;
use crate::indicators::parabolic_sar::parabolic_sar_store;
use crate::indicators::performance_index::performance_index_store;
use crate::indicators::pivot_points::pivot_points_store;
use crate::indicators::positive_volume_index::positive_volume_index_store;
use crate::indicators::pretty_good_oscillator::pretty_good_oscillator_store;
use crate::indicators::price_momentum_oscillator::price_momentum_oscillator_store;
use crate::indicators::price_oscillator::price_oscillator_store;
use crate::indicators::price_relative::price_relative_store;
use crate::indicators::price_volume_trend::price_volume_trend_store;
use crate::indicators::prime_number_bands::prime_number_bands_store;
use crate::indicators::prime_number_oscillator::prime_number_oscillator_store;
use crate::indicators::projected_aggregate_volume::projected_aggregate_volume_store;
use crate::indicators::projected_volume_at_time::projected_volume_at_time_store;
use crate::indicators::psychological_line::psychological_line_store;
use crate::indicators::qstick::qstick_store;
use crate::indicators::rainbow_ma::rainbow_ma_store;
use crate::indicators::rainbow_oscillator::rainbow_oscillator_store;
use crate::indicators::random_walk_index::random_walk_index_store;
use crate::indicators::ravi::ravi_store;
use crate::indicators::relative_vigor::relative_vigor_store;
use crate::indicators::relative_volatility::relative_volatility_store;
use crate::indicators::roc::roc_store;
use crate::indicators::rsi::rsi_outputs_store;
use crate::indicators::schaff_trend_cycle::schaff_trend_cycle_store;
use crate::indicators::shinohara_intensity::shinohara_intensity_store;
use crate::indicators::sma::sma_close_store;
use crate::indicators::stddev::stddev_store;
use crate::indicators::stoch::stochastic_store;
use crate::indicators::stoch_rsi::stoch_rsi_store;
use crate::indicators::stochastic_momentum::stochastic_momentum_store;
use crate::indicators::supertrend::supertrend_store;
use crate::indicators::swing_index::swing_index_store;
use crate::indicators::tema::tema_store;
use crate::indicators::trade_volume_index::trade_volume_index_store;
use crate::indicators::trend_intensity::trend_intensity_store;
use crate::indicators::trima::trima_store;
use crate::indicators::trix::trix_store;
use crate::indicators::true_range::true_range_series_store;
use crate::indicators::tsi::tsi_store;
use crate::indicators::twiggs_money_flow::twiggs_money_flow_store;
use crate::indicators::typical_price::typical_price_store;
use crate::indicators::ulcer_index::ulcer_index_store;
use crate::indicators::ultimate_oscillator::ultimate_oscillator_store;
use crate::indicators::valuation_lines::valuation_lines_store;
use crate::indicators::vertical_horizontal_filter::vertical_horizontal_filter_store;
use crate::indicators::volume_chart::volume_chart_store;
use crate::indicators::volume_oscillator::volume_oscillator_store;
use crate::indicators::volume_profile::volume_profile_store;
use crate::indicators::volume_roc::volume_roc_store;
use crate::indicators::volume_underlay::volume_underlay_store;
use crate::indicators::vortex_indicator::vortex_indicator_store;
use crate::indicators::vwap::vwap_store;
use crate::indicators::vwma::vwma_store;
use crate::indicators::weighted_close::weighted_close_store;
use crate::indicators::williams_ad::williams_ad_store;
use crate::indicators::williams_r::williams_r_store;
use crate::indicators::wma::wma_store;
use crate::indicators::zigzag::zigzag_store;

#[allow(clippy::too_many_arguments)]
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
    anchor: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    match kind {
        "SMA" => rc_one_output(sma_close_store(store, period, nodes)),
        "EMA" => rc_one_output(ema_close_store(store, period, nodes)),
        "RSI" => rsi_outputs_store(store, period, nodes),
        "ROC" => rc_one_output(roc_store(store, period, nodes)),
        "CCI" => rc_one_output(cci_store(store, period, nodes)),
        "MFI" => rc_one_output(mfi_store(store, period, nodes)),
        "CMF" => rc_one_output(cmf_store(store, period, nodes)),
        "WILLIAMS_R" => rc_one_output(williams_r_store(store, period, nodes)),
        "OBV" => rc_one_output(obv_store(store, nodes)),
        "ADL" => rc_one_output(adl_store(store, nodes)),
        "VWAP" => vwap_store(store, nodes),
        "VWMA" => rc_one_output(vwma_store(store, period, nodes)),
        "WILLIAMS_AD" => rc_one_output(williams_ad_store(store, nodes)),
        "ATR" => rc_one_output(atr_store(store, period, nodes)),
        "ADX" => adx_store(store, period, nodes),
        "SUPERTREND" => supertrend_store(store, period, multiplier, nodes),
        "KELTNER" => keltner_store(store, period, multiplier, nodes),
        "STARC" => starc_store(store, period, multiplier, nodes),
        "WMA" => rc_one_output(wma_store(store, period, nodes)),
        "HMA" => rc_one_output(hma_store(store, period, nodes)),
        "LINEAR_REGRESSION" => rc_one_output(linear_regression_store(store, period, nodes)),
        "LINEAR_REG_FORECAST" => rc_one_output(linear_reg_forecast_store(store, period, nodes)),
        "LINEAR_REG_INTERCEPT" => rc_one_output(linear_reg_intercept_store(store, period, nodes)),
        "DEMA" => rc_one_output(dema_store(store, period, nodes)),
        "TEMA" => rc_one_output(tema_store(store, period, nodes)),
        "TRIMA" => rc_one_output(trima_store(store, period, nodes)),
        "STDDEV" => rc_one_output(stddev_store(store, period, nodes)),
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
        "ULTIMATE_OSCILLATOR" => rc_one_output(ultimate_oscillator_store(store, period, stoch_period, smooth, nodes)),
        "CHAIKIN_VOLATILITY" => rc_one_output(chaikin_volatility_store(store, period, nodes)),
        "STOCH_RSI" => stoch_rsi_store(store, period, stoch_period, smooth, signal, nodes),
        "CHAIKIN_OSCILLATOR" => rc_one_output(chaikin_oscillator_store(
            store, macd_params.unwrap_or(MacdParams { fast: 3, slow: 10, signal: 9 }), nodes,
        )),
        "MACD" => macd_store(store, macd_params.unwrap_or(MacdParams { fast: 12, slow: 26, signal: 9 }), nodes),
        "PPO" => ppo_store(store, macd_params.unwrap_or(MacdParams { fast: 12, slow: 26, signal: 9 }), nodes),
        "MEDIAN_PRICE" => rc_one_output(median_price_store(store, nodes)),
        "HIGHEST_HIGH" => rc_one_output(highest_high_store(store, period, nodes)),
        "LOWEST_LOW" => rc_one_output(lowest_low_store(store, period, nodes)),
        "ALLIGATOR" => alligator_store(store, nodes),
        "ATR_BANDS" => atr_bands_store(store, period, multiplier, nodes),
        "HIGH_LOW_BANDS" => high_low_bands_store(store, period, nodes),
        "FRACTAL_CHAOS_BANDS" => fractal_chaos_bands_store(store, nodes),
        "GMMA" => gmma_store(store, nodes),
        "ANCHORED_VWAP" => anchored_vwap_store(store, anchor, nodes),
        "TYPICAL_PRICE" => rc_one_output(typical_price_store(store, nodes)),
        "WEIGHTED_CLOSE" => rc_one_output(weighted_close_store(store, nodes)),
        "MA_CROSS" => ma_cross_store(store, macd_params.map_or(period, |m| m.fast), macd_params.map_or(stoch_period, |m| m.slow), nodes),
        "RAINBOW_MA" => rainbow_ma_store(store, period, nodes),
        "PRIME_NUMBER_BANDS" => prime_number_bands_store(store, nodes),
        "TIME_SERIES_FORECAST" => rc_one_output(linear_reg_forecast_store(store, period, nodes)),
        "VALUATION_LINES" => valuation_lines_store(store, period, multiplier, nodes),
        "BETA" => rc_one_output(beta_store(store, period, nodes)),
        "CORRELATION_COEFFICIENT" => rc_one_output(correlation_coefficient_store(store, period, nodes)),
        "PERFORMANCE_INDEX" => rc_one_output(performance_index_store(store, nodes)),
        "PRICE_RELATIVE" => rc_one_output(price_relative_store(store, period, nodes)),
        "AWESOME_OSCILLATOR" => rc_one_output(awesome_oscillator_store(store, nodes)),
        "BOLLINGER_PCT_B" => rc_one_output(bollinger_pct_b_store(store, period, multiplier, nodes)),
        "CENTER_OF_GRAVITY" => rc_one_output(center_of_gravity_store(store, period, nodes)),
        "CHANDE_FORECAST" => rc_one_output(chande_forecast_store(store, period, nodes)),
        "CHANDE_MOMENTUM" => rc_one_output(chande_momentum_store(store, period, nodes)),
        "COPPOCK_CURVE" => rc_one_output(coppock_curve_store(store, nodes)),
        "DISPARITY_INDEX" => rc_one_output(disparity_index_store(store, period, nodes)),
        "EASE_OF_MOVEMENT" => rc_one_output(ease_of_movement_store(store, period, nodes)),
        "EHLER_FISHER" => ehler_fisher_store(store, period, nodes),
        "ELDER_RAY" => elder_ray_store(store, period, nodes),
        "FRACTAL_CHAOS_OSCILLATOR" => rc_one_output(fractal_chaos_oscillator_store(store, nodes)),
        "GATOR_OSCILLATOR" => gator_oscillator_store(store, nodes),
        "INTRADAY_MOMENTUM" => rc_one_output(intraday_momentum_store(store, period, nodes)),
        "LINEAR_REG_SLOPE" => rc_one_output(linear_reg_slope_store(store, period, nodes)),
        "MA_DEVIATION" => rc_one_output(ma_deviation_store(store, period, nodes)),
        "PRETTY_GOOD_OSCILLATOR" => rc_one_output(pretty_good_oscillator_store(store, period, nodes)),
        "PRICE_MOMENTUM_OSCILLATOR" => rc_one_output(price_momentum_oscillator_store(store, period, smooth, nodes)),
        "PRICE_OSCILLATOR" => rc_one_output(price_oscillator_store(store, macd_params.unwrap_or(MacdParams { fast: 12, slow: 26, signal: 9 }), nodes)),
        "RAINBOW_OSCILLATOR" => rc_one_output(rainbow_oscillator_store(store, period, nodes)),
        "RAVI" => rc_one_output(ravi_store(store, period, stoch_period, nodes)),
        "RELATIVE_VIGOR" => relative_vigor_store(store, period, nodes),
        "SCHAFF_TREND_CYCLE" => rc_one_output(schaff_trend_cycle_store(store, macd_params.map_or(12, |m| m.fast), macd_params.map_or(26, |m| m.slow), stoch_period, nodes)),
        "STOCHASTIC_MOMENTUM" => rc_one_output(stochastic_momentum_store(store, period, smooth, nodes)),
        "SWING_INDEX" => rc_one_output(swing_index_store(store, nodes)),
        "TREND_INTENSITY" => rc_one_output(trend_intensity_store(store, period, nodes)),
        "VOLUME_OSCILLATOR" => rc_one_output(volume_oscillator_store(store, macd_params.unwrap_or(MacdParams { fast: 5, slow: 10, signal: 9 }), nodes)),
        "KLINGER_VOLUME" => rc_one_output(klinger_volume_store(store, nodes)),
        "MARKET_FACILITATION" => rc_one_output(market_facilitation_store(store, nodes)),
        "NEGATIVE_VOLUME_INDEX" => rc_one_output(negative_volume_index_store(store, nodes)),
        "POSITIVE_VOLUME_INDEX" => rc_one_output(positive_volume_index_store(store, nodes)),
        "PRICE_VOLUME_TREND" => rc_one_output(price_volume_trend_store(store, nodes)),
        "TRADE_VOLUME_INDEX" => rc_one_output(trade_volume_index_store(store, nodes)),
        "TWIGGS_MONEY_FLOW" => rc_one_output(twiggs_money_flow_store(store, period, nodes)),
        "PROJECTED_AGGREGATE_VOLUME" => rc_one_output(projected_aggregate_volume_store(store, period, nodes)),
        "PROJECTED_VOLUME_AT_TIME" => rc_one_output(projected_volume_at_time_store(store, period, nodes)),
        "HISTORICAL_VOLATILITY" => rc_one_output(historical_volatility_store(store, period, nodes)),
        "LINEAR_REG_R2" => rc_one_output(linear_reg_r2_store(store, period, nodes)),
        "PRIME_NUMBER_OSCILLATOR" => rc_one_output(prime_number_oscillator_store(store, nodes)),
        "RANDOM_WALK_INDEX" => random_walk_index_store(store, period, nodes),
        "DARVAS_BOX" => darvas_box_store(store, nodes),
        "VOLUME_PROFILE" => volume_profile_store(store, period, nodes),
        "CHOPPINESS_INDEX" => rc_one_output(choppiness_index_store(store, period, nodes)),
        "ELDER_IMPULSE" => rc_one_output(elder_impulse_store(store, period, nodes)),
        "GONOGO_TREND" => rc_one_output(gonogo_trend_store(store, period, nodes)),
        "PSYCHOLOGICAL_LINE" => rc_one_output(psychological_line_store(store, period, nodes)),
        "QSTICK" => rc_one_output(qstick_store(store, period, nodes)),
        "SHINOHARA_INTENSITY" => shinohara_intensity_store(store, period, nodes),
        "ULCER_INDEX" => rc_one_output(ulcer_index_store(store, period, nodes)),
        "VERTICAL_HORIZONTAL_FILTER" => rc_one_output(vertical_horizontal_filter_store(store, period, nodes)),
        "VORTEX_INDICATOR" => vortex_indicator_store(store, period, nodes),
        "ZIGZAG" => rc_one_output(zigzag_store(store, multiplier, nodes)),
        "BOLLINGER_BANDWIDTH" => rc_one_output(bollinger_bandwidth_store(store, period, multiplier, nodes)),
        "DONCHIAN_WIDTH" => rc_one_output(donchian_width_store(store, period, nodes)),
        "GOPALAKRISHNAN_RANGE" => rc_one_output(gopalakrishnan_range_store(store, period, nodes)),
        "HIGH_MINUS_LOW" => rc_one_output(high_minus_low_store(store, nodes)),
        "MASS_INDEX" => rc_one_output(mass_index_store(store, period, nodes)),
        "RELATIVE_VOLATILITY" => rc_one_output(relative_volatility_store(store, period, nodes)),
        "TRUE_RANGE" => rc_one_output(true_range_series_store(store, nodes)),
        "VOLUME_CHART" => rc_one_output(volume_chart_store(store, nodes)),
        "VOLUME_ROC" => rc_one_output(volume_roc_store(store, period, nodes)),
        "VOLUME_UNDERLAY" => rc_one_output(volume_underlay_store(store, nodes)),
        _ => Vec::new(),
    }
}
